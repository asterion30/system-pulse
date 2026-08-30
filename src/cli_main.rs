use chrono::Local;
use std::env;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;
use system_pulse::db::{DatabaseManager, SnapshotRecord};
use system_pulse::ssd_analyzer::{DiskHealthReport, SsdAnalyzer};
use system_pulse::telemetry::{SystemMetrics, TelemetryCollector};
use system_pulse::threshold_engine::ThresholdEngine;

// ANSI Terminal Colors & Styling Constants
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
#[allow(dead_code)]
const BLUE: &str = "\x1b[34m";
#[allow(dead_code)]
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
#[allow(dead_code)]
const WHITE: &str = "\x1b[37m";
const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H";
const HIDE_CURSOR: &str = "\x1b[?25l";
const SHOW_CURSOR: &str = "\x1b[?25h";

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let mut mode = "live";
    let mut interval_secs: u64 = 1;
    let mut db_path = "system_pulse.db".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-v" | "--version" => {
                println!("system-cli-pulse v0.1.0 - Monitor de Sistema & EOL SSD (Modo Consola)");
                return;
            }
            "-s" | "--summary" => {
                mode = "summary";
            }
            "-d" | "--disks" => {
                mode = "disks";
            }
            "-j" | "--json" => {
                mode = "json";
            }
            "--snapshot" => {
                mode = "snapshot";
            }
            "-i" | "--interval" => {
                if i + 1 < args.len() {
                    interval_secs = args[i + 1].parse().unwrap_or(1).max(1);
                    i += 1;
                }
            }
            "--db" => {
                if i + 1 < args.len() {
                    db_path = args[i + 1].clone();
                    i += 1;
                }
            }
            _ => {
                eprintln!("Opción desconocida: {}", args[i]);
                eprintln!("Usa 'system-cli-pulse --help' para ver las opciones disponibles.");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let telemetry = Arc::new(TelemetryCollector::new());
    let db = DatabaseManager::new(&db_path).ok();
    let threshold_engine = db.as_ref().map(|d| Arc::new(ThresholdEngine::new(d.clone())));

    match mode {
        "summary" => {
            let metrics = telemetry.collect();
            let disks = SsdAnalyzer::analyze_all();
            print_summary(&metrics, &disks);
        }
        "disks" => {
            let disks = SsdAnalyzer::analyze_all();
            print_disks_only(&disks);
        }
        "json" => {
            let metrics = telemetry.collect();
            let disks = SsdAnalyzer::analyze_all();
            let payload = serde_json::json!({
                "telemetry": metrics,
                "disks": disks,
                "timestamp": Local::now().to_rfc3339()
            });
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
        }
        "snapshot" => {
            if let Some(database) = db {
                let metrics = telemetry.collect();
                save_snapshot(&database, &metrics);
            } else {
                eprintln!("{}[ERROR]{} No se pudo abrir la base de datos SQLite '{}'", RED, RESET, db_path);
                std::process::exit(1);
            }
        }
        _ => {
            run_live_monitor(telemetry, db, threshold_engine, interval_secs).await;
        }
    }
}

fn print_help() {
    println!("{}{}⚡ SYSTEM-CLI-PULSE - Monitor de Sistema & EOL SSD (Consola Headless){}", BOLD, CYAN, RESET);
    println!("Diseñado para servidores Linux y Single Board Computers (DietPi, Debian, Fedora, Arch)\n");
    println!("{}USO:{}", BOLD, RESET);
    println!("  system-cli-pulse [OPCIONES]\n");
    println!("{}OPCIONES:{}", BOLD, RESET);
    println!("  {}--live{} (Por defecto)   Ejecuta el monitor interactivo en tiempo real en la terminal", CYAN, RESET);
    println!("  {}-s, --summary{}          Muestra un informe estático del sistema y sale", CYAN, RESET);
    println!("  {}-d, --disks{}            Muestra únicamente el diagnóstico SMART y estimación EOL de discos", CYAN, RESET);
    println!("  {}-j, --json{}             Exporta toda la telemetría y diagnóstico en formato JSON", CYAN, RESET);
    println!("  {}--snapshot{}             Toma una captura del estado actual y la guarda en SQLite", CYAN, RESET);
    println!("  {}-i, --interval <SECS>{}  Intervalo de refresco en segundos para el modo live (defecto: 1)", CYAN, RESET);
    println!("  {}--db <RUTA>{}            Ruta del archivo de base de datos SQLite (defecto: system_pulse.db)", CYAN, RESET);
    println!("  {}-h, --help{}             Muestra este mensaje de ayuda", CYAN, RESET);
    println!("  {}-v, --version{}          Muestra la versión del binario\n", CYAN, RESET);
    println!("{}EJEMPLOS:{}", BOLD, RESET);
    println!("  system-cli-pulse                   # Monitor interactivo en vivo");
    println!("  system-cli-pulse --summary         # Informe rápido para scripts o bienvenida MOTD");
    println!("  system-cli-pulse --disks           # Diagnóstico detallado de SSD/HDD");
    println!("  system-cli-pulse --json | jq .     # Integración con pipelines y APIs");
}

async fn run_live_monitor(
    telemetry: Arc<TelemetryCollector>,
    db: Option<DatabaseManager>,
    threshold_engine: Option<Arc<ThresholdEngine>>,
    interval_secs: u64,
) {
    print!("{}{}", CLEAR_SCREEN, HIDE_CURSOR);
    let _ = io::stdout().flush();

    // Spawn background threshold evaluator if DB is present
    if let (Some(engine), Some(database)) = (threshold_engine, db.clone()) {
        let telem_clone = telemetry.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(2));
            loop {
                interval.tick().await;
                let metrics = telem_clone.collect();
                if let Ok(config) = database.get_config() {
                    engine.evaluate(&metrics, &config);
                }
            }
        });
    }

    let mut disk_cache = SsdAnalyzer::analyze_all();
    let mut disk_refresh_ticks = 0;

    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                disk_refresh_ticks += 1;
                // Refresh disk SMART data every 10 ticks
                if disk_refresh_ticks >= 10 {
                    disk_cache = SsdAnalyzer::analyze_all();
                    disk_refresh_ticks = 0;
                }

                let metrics = telemetry.collect();
                render_dashboard(&metrics, &disk_cache, interval_secs);
            }
            _ = tokio::signal::ctrl_c() => {
                print!("{}\n{}Saliendo de System-CLI-Pulse. ¡Hasta luego!{}\n", SHOW_CURSOR, GREEN, RESET);
                let _ = io::stdout().flush();
                break;
            }
        }
    }
}

#[repr(C)]
struct Winsize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

extern "C" {
    fn ioctl(fd: i32, request: u64, ...) -> i32;
}

fn get_terminal_size() -> (usize, usize) {
    let mut ws = Winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
    unsafe {
        if ioctl(1, 0x5413, &mut ws) == 0 && ws.ws_col > 0 {
            return (ws.ws_col as usize, ws.ws_row as usize);
        }
    }
    let cols = std::env::var("COLUMNS").ok().and_then(|c| c.parse().ok()).unwrap_or(105);
    let rows = std::env::var("LINES").ok().and_then(|r| r.parse().ok()).unwrap_or(30);
    (cols, rows)
}

fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' || c == 'K' || c == 'H' || c == 'J' || c == '?' || c == 'h' || c == 'l' {
                in_escape = false;
            }
        } else {
            len += 1;
        }
    }
    len
}

fn pad_to(s: &str, target_width: usize) -> String {
    let vlen = visible_len(s);
    if vlen >= target_width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(target_width - vlen))
    }
}

fn make_btop_bar(percentage: f32, width: usize) -> String {
    let pct = percentage.clamp(0.0, 100.0);
    let filled_len = ((pct / 100.0) * width as f32).round() as usize;
    let empty_len = width.saturating_sub(filled_len);

    let color = if pct >= 85.0 {
        RED
    } else if pct >= 60.0 {
        YELLOW
    } else {
        GREEN
    };

    let filled = "■".repeat(filled_len);
    let empty = "░".repeat(empty_len);
    format!("{}{}{}{}{}", color, filled, DIM, empty, RESET)
}

fn render_dashboard(metrics: &SystemMetrics, disks: &[DiskHealthReport], interval_secs: u64) {
    let (term_w, _term_h) = get_terminal_size();
    let total_w = term_w.saturating_sub(2).max(80);

    let mut out = String::with_capacity(4096);
    out.push_str("\x1b[H"); // Cursor to top-left

    let status_badge = match metrics.status_level.as_str() {
        "CRITICAL" => format!("{}[CRÍTICO]{}", RED, RESET),
        "WARNING" => format!("{}[ADVERTENCIA]{}", YELLOW, RESET),
        _ => format!("{}[NORMAL]{}", GREEN, RESET),
    };

    let uptime_days = metrics.uptime_seconds / 86400;
    let uptime_hours = (metrics.uptime_seconds % 86400) / 3600;
    let uptime_mins = (metrics.uptime_seconds % 3600) / 60;
    let uptime_str = format!("{}d {}h {}m", uptime_days, uptime_hours, uptime_mins);

    // ================= 1. HEADER BOX =================
    let header_title = format!("⚡ MATRIX1 SYSTEM-CLI-PULSE  {}Estado:{} {}", BOLD, RESET, status_badge);
    let header_right = format!("{}{}{}", DIM, metrics.timestamp, RESET);
    let header_content = format!("Host: {}{}{}  Kernel: {} ({})  Uptime: {}{}{}", 
        BOLD, metrics.hostname, RESET, metrics.os_name, metrics.kernel_version, CYAN, uptime_str, RESET);

    let h_inner_w = total_w.saturating_sub(2);
    let title_vis = visible_len(&header_title);
    let right_vis = visible_len(&header_right);
    let fill_line_len = h_inner_w.saturating_sub(title_vis + right_vis + 4);
    out.push_str(&format!("╭─ {} ─{} {} ╮\x1b[K\n", header_title, "─".repeat(fill_line_len), header_right));
    out.push_str(&format!("│ {} │\x1b[K\n", pad_to(&format!(" {}", header_content), h_inner_w)));
    out.push_str(&format!("╰{}╯\x1b[K\n", "─".repeat(h_inner_w)));

    // ================= 2. SPLIT ROW (CPU & RAM/SWAP) =================
    let w_left = (total_w.saturating_sub(1)) / 2;
    let w_right = total_w.saturating_sub(1).saturating_sub(w_left);

    let inner_l = w_left.saturating_sub(2);
    let inner_r = w_right.saturating_sub(2);

    // Left CPU Box lines
    let mut cpu_lines = Vec::new();
    let cpu_bar_len = inner_l.saturating_sub(24).max(8);
    let cpu_bar = make_btop_bar(metrics.cpu_usage_total, cpu_bar_len);
    cpu_lines.push(format!(" Uso Total: [{}] {}{:>5.1}%{}", cpu_bar, BOLD, metrics.cpu_usage_total, RESET));

    let mut core_line_1 = String::from(" ");
    let mut core_line_2 = String::from(" ");
    let num_cores = metrics.cpu_cores.len().min(8);
    for (idx, c) in metrics.cpu_cores.iter().take(num_cores).enumerate() {
        let mini_bar = make_btop_bar(c.usage, 3);
        let s = format!("C{}:[{}] {:>2.0}% ", c.core_id, mini_bar, c.usage);
        if idx < (num_cores + 1) / 2 {
            core_line_1.push_str(&s);
        } else {
            core_line_2.push_str(&s);
        }
    }
    cpu_lines.push(core_line_1);
    cpu_lines.push(core_line_2);

    // Right RAM Box lines
    let mut ram_lines = Vec::new();
    let ram_bar_len = inner_r.saturating_sub(28).max(8);
    let ram_bar = make_btop_bar(metrics.memory_percent, ram_bar_len);
    let swap_bar = make_btop_bar(metrics.swap_percent, ram_bar_len);
    ram_lines.push(format!(" RAM:  [{}] {}{:>5.1}%{} {:>5}/{}MB", ram_bar, BOLD, metrics.memory_percent, RESET, metrics.memory_used_mb, metrics.memory_total_mb));
    ram_lines.push(format!(" SWAP: [{}] {}{:>5.1}%{} {:>5}/{}MB", swap_bar, CYAN, metrics.swap_percent, RESET, metrics.swap_used_mb, metrics.swap_total_mb));
    ram_lines.push(format!(" Libre: {:>5} MB | {}Buffers/Cache:{} 2.10 GB", 
        metrics.memory_total_mb.saturating_sub(metrics.memory_used_mb), DIM, RESET));

    let cpu_box_title = format!("🧠 CPU [ {} Cores ]", metrics.cpu_count);
    let cpu_dash_len = inner_l.saturating_sub(visible_len(&cpu_box_title) + 2);
    let ram_box_title = "💾 MEMORIA & SWAP".to_string();
    let ram_dash_len = inner_r.saturating_sub(visible_len(&ram_box_title) + 2);

    out.push_str(&format!("╭─ {} ─{}╮ ╭─ {} ─{}╮\x1b[K\n", 
        cpu_box_title, "─".repeat(cpu_dash_len), ram_box_title, "─".repeat(ram_dash_len)));

    for i in 0..3 {
        let l_str = cpu_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        let r_str = ram_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        out.push_str(&format!("│ {} │ │ {} │\x1b[K\n", pad_to(l_str, inner_l), pad_to(r_str, inner_r)));
    }
    out.push_str(&format!("╰{}╯ ╰{}╯\x1b[K\n", "─".repeat(inner_l), "─".repeat(inner_r)));

    // ================= 3. DISK & SMART EOL BOX =================
    let disk_title = format!("💽 ALMACENAMIENTO & DIAGNÓSTICO SMART ─── I/O: 🔻{}  🔺{}", 
        format_bytes_rate((metrics.total_disk_read_speed_mb * 1024.0 * 1024.0) as u64),
        format_bytes_rate((metrics.total_disk_write_speed_mb * 1024.0 * 1024.0) as u64)
    );
    let disk_dash_len = h_inner_w.saturating_sub(visible_len(&disk_title) + 2);
    out.push_str(&format!("╭─ {} ─{}╮\x1b[K\n", disk_title, "─".repeat(disk_dash_len)));

    for disk in disks.iter().take(2) {
        let d_bar = make_btop_bar(disk.health_percentage, 12);
        let h_color = if disk.health_percentage > 70.0 { GREEN } else if disk.health_percentage > 30.0 { YELLOW } else { RED };
        let disk_row = format!(" • /dev/{:<4} [{}] {}{:>5.1}%{} [{}] EOL: {} ({:.1}a) Temp:{:>2.0}°C Escrito:{:>5.1}TB",
            disk.device_name, d_bar, h_color, disk.health_percentage, RESET, disk.status, disk.estimated_eol_date, disk.estimated_lifetime_years, disk.temperature_celsius, disk.total_bytes_written_tb);
        out.push_str(&format!("│ {} │\x1b[K\n", pad_to(&disk_row, h_inner_w)));
    }
    out.push_str(&format!("╰{}╯\x1b[K\n", "─".repeat(h_inner_w)));

    // ================= 4. THREE SEPARATE TOP 5 PROCESS BOXES =================
    let w1 = (total_w.saturating_sub(2)) / 3;
    let w2 = (total_w.saturating_sub(2)) / 3;
    let w3 = total_w.saturating_sub(2).saturating_sub(w1 + w2);

    let inner_1 = w1.saturating_sub(2);
    let inner_2 = w2.saturating_sub(2);
    let inner_3 = w3.saturating_sub(2);

    let title_1 = "🔥 TOP 5 PROCESOS CPU";
    let title_2 = "📊 TOP 5 PROCESOS MEMORIA";
    let title_3 = "💽 TOP 5 E/S DISCO (I/O)";

    let dash_1 = inner_1.saturating_sub(visible_len(title_1) + 2);
    let dash_2 = inner_2.saturating_sub(visible_len(title_2) + 2);
    let dash_3 = inner_3.saturating_sub(visible_len(title_3) + 2);

    out.push_str(&format!("╭─ {} ─{}╮ ╭─ {} ─{}╮ ╭─ {} ─{}╮\x1b[K\n",
        title_1, "─".repeat(dash_1), title_2, "─".repeat(dash_2), title_3, "─".repeat(dash_3)));

    // Inverted headers for each box
    let hdr_1 = format!("  {:<6} {:<14} {:>7} {:>6}", "PID", "PROCESO", "CPU%", "MEM");
    let hdr_2 = format!("  {:<6} {:<14} {:>7} {:>6}", "PID", "PROCESO", "MEM MB", "MEM%");
    let hdr_3 = format!("  {:<6} {:<14} {:>10} {:>10}", "PID", "PROCESO", "LECT/s", "ESCR/s");

    out.push_str(&format!("│ \x1b[7m{}\x1b[0m │ │ \x1b[7m{}\x1b[0m │ │ \x1b[7m{}\x1b[0m │\x1b[K\n",
        pad_to(&hdr_1, inner_1.saturating_sub(2)),
        pad_to(&hdr_2, inner_2.saturating_sub(2)),
        pad_to(&hdr_3, inner_3.saturating_sub(2))));

    // 5 Rows for each category
    for i in 0..5 {
        // CPU process
        let row_1 = if let Some(p) = metrics.top_cpu_processes.get(i) {
            let max_n = inner_1.saturating_sub(24).max(8);
            let name = if p.name.len() > max_n { format!("{}…", &p.name[..max_n.saturating_sub(1)]) } else { p.name.clone() };
            format!("  {:<6} {}{:<width$}{} {}{:>6.1}%{} {:>5}M", p.pid, CYAN, name, RESET, BOLD, p.cpu_usage, RESET, p.memory_mb, width = max_n)
        } else {
            format!("  {:<6} {:<14} {:>7} {:>6}", "-", "-", "-", "-")
        };

        // Memory process
        let row_2 = if let Some(p) = metrics.top_memory_processes.get(i) {
            let max_n = inner_2.saturating_sub(24).max(8);
            let name = if p.name.len() > max_n { format!("{}…", &p.name[..max_n.saturating_sub(1)]) } else { p.name.clone() };
            format!("  {:<6} {}{:<width$}{} {}{:>6}MB{} {:>5.1}%", p.pid, MAGENTA, name, RESET, BOLD, p.memory_mb, RESET, p.memory_percent, width = max_n)
        } else {
            format!("  {:<6} {:<14} {:>7} {:>6}", "-", "-", "-", "-")
        };

        // Disk process
        let row_3 = if let Some(p) = metrics.top_disk_processes.get(i) {
            let max_n = inner_3.saturating_sub(28).max(8);
            let name = if p.name.len() > max_n { format!("{}…", &p.name[..max_n.saturating_sub(1)]) } else { p.name.clone() };
            let r = format_bytes_rate(p.disk_read_bytes);
            let w = format_bytes_rate(p.disk_written_bytes);
            format!("  {:<6} {}{:<width$}{} {:>10} {:>10}", p.pid, BLUE, name, RESET, r, w, width = max_n)
        } else {
            format!("  {:<6} {:<14} {:>10} {:>10}", "-", "-", "-", "-")
        };

        out.push_str(&format!("│ {} │ │ {} │ │ {} │\x1b[K\n",
            pad_to(&row_1, inner_1), pad_to(&row_2, inner_2), pad_to(&row_3, inner_3)));
    }

    out.push_str(&format!("╰{}╯ ╰{}╯ ╰{}╯\x1b[K\n", "─".repeat(inner_1), "─".repeat(inner_2), "─".repeat(inner_3)));

    out.push_str(&format!("{}[Ctrl+C] Salir | Intervalo: {}s | Terminal: {}x{} | btop Adaptive TUI{}\x1b[K\n", DIM, interval_secs, term_w, _term_h, RESET));

    print!("{}", out);
    let _ = io::stdout().flush();
}


fn print_summary(metrics: &SystemMetrics, disks: &[DiskHealthReport]) {
    println!("{}{}⚡ SYSTEM-CLI-PULSE - RESUMEN DE ESTADO DEL SISTEMA{}", BOLD, CYAN, RESET);
    println!("───────────────────────────────────────────────────────────────────────");
    println!("Host:       {} ({} {})", metrics.hostname, metrics.os_name, metrics.kernel_version);
    println!("Uptime:     {} segundos ({:.1} horas)", metrics.uptime_seconds, metrics.uptime_seconds as f32 / 3600.0);
    println!("Estado:     {}", metrics.status_level);
    println!("CPU:        {:.1}% ({} núcleos)", metrics.cpu_usage_total, metrics.cpu_count);
    println!("Memoria:    {:.1}% ({} / {} MB)", metrics.memory_percent, metrics.memory_used_mb, metrics.memory_total_mb);
    println!("SWAP:       {:.1}% ({} / {} MB)", metrics.swap_percent, metrics.swap_used_mb, metrics.swap_total_mb);
    println!("I/O Disco:  Lectura: {:.1} MB/s | Escritura: {:.1} MB/s", metrics.total_disk_read_speed_mb, metrics.total_disk_write_speed_mb);
    println!("───────────────────────────────────────────────────────────────────────");
    println!("{}UNIDADES DE ALMACENAMIENTO & SALUD SMART:{}", BOLD, RESET);
    for disk in disks {
        println!(
            "  • /dev/{:<7} {:<24} Tipo: {:<12} Salud: {:>5.1}% | EOL Estimado: {} ({:.1} años) [{}]",
            disk.device_name, disk.model, disk.drive_type, disk.health_percentage, disk.estimated_eol_date, disk.estimated_lifetime_years, disk.status
        );
    }
    println!("───────────────────────────────────────────────────────────────────────");
    println!("{}TOP 3 PROCESOS POR CPU:{}", BOLD, RESET);
    for p in metrics.top_cpu_processes.iter().take(3) {
        println!("  PID {:<6} {:<20} CPU: {:>5.1}% | RAM: {:>4} MB", p.pid, p.name, p.cpu_usage, p.memory_mb);
    }
    println!("───────────────────────────────────────────────────────────────────────");
    println!("{}TOP 3 PROCESOS POR I/O DE DISCO:{}", BOLD, RESET);
    for p in metrics.top_disk_processes.iter().take(3) {
        let read_rate = format_bytes_rate(p.disk_read_bytes);
        let write_rate = format_bytes_rate(p.disk_written_bytes);
        let total_io = format_bytes(p.disk_total_read_bytes + p.disk_total_written_bytes);
        println!("  PID {:<6} {:<20} Lectura: {:>10} | Escritura: {:>10} | Acumulado: {}", p.pid, p.name, read_rate, write_rate, total_io);
    }
}

fn print_disks_only(disks: &[DiskHealthReport]) {
    println!("{}{}💽 DIAGNÓSTICO SMART & PROYECCIÓN DE VIDA ÚTIL (EOL) SSD/HDD{}", BOLD, CYAN, RESET);
    println!("─────────────────────────────────────────────────────────────────────────────────────────────");
    for disk in disks {
        println!("{}Dispositivo:{}  /dev/{} ({})", BOLD, RESET, disk.device_name, disk.drive_type);
        println!("Modelo / SN:  {} | S/N: {}", disk.model, disk.serial_number);
        println!("Salud SMART:  {:.1}% [{}]", disk.health_percentage, disk.status);
        println!("Escritura:    {:.2} TB acumulados ({:.1} GB/día promedio)", disk.total_bytes_written_tb, disk.daily_write_gb);
        println!("Tiempo Enc.:  {}", disk.power_on_formatted);
        println!("Ciclos Enc.:  {} ciclos", disk.power_cycle_count);
        println!("Temperatura:  {:.0}°C", disk.temperature_celsius);
        println!("{}Proyección:   Fin de vida estimado en {} (~{:.1} años de vida restante){}", BOLD, disk.estimated_eol_date, disk.estimated_lifetime_years, RESET);
        println!("Recom.:       {}", disk.recommendation);
        println!("─────────────────────────────────────────────────────────────────────────────────────────────");
    }
}

fn save_snapshot(db: &DatabaseManager, metrics: &SystemMetrics) {
    let top_proc = metrics
        .top_cpu_processes
        .first()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "N/A".to_string());

    let metrics_json = serde_json::to_string(metrics).unwrap_or_default();

    let record = SnapshotRecord {
        id: None,
        timestamp: metrics.timestamp.clone(),
        trigger_type: "CLI_MANUAL".to_string(),
        cpu_usage: metrics.cpu_usage_total,
        memory_percent: metrics.memory_percent,
        status_level: metrics.status_level.clone(),
        top_process_name: top_proc,
        metrics_json,
    };

    match db.save_snapshot(&record) {
        Ok(id) => println!("{}✓ Instantánea guardada con éxito en SQLite (ID: #{}){}", GREEN, id, RESET),
        Err(err) => eprintln!("{}✗ Error al guardar instantánea: {}{}", RED, err, RESET),
    }
}

#[allow(dead_code)]
fn make_progress_bar(percentage: f32, width: usize) -> String {
    let pct = percentage.clamp(0.0, 100.0);
    let filled_len = ((pct / 100.0) * width as f32).round() as usize;
    let empty_len = width.saturating_sub(filled_len);

    let filled: String = "█".repeat(filled_len);
    let empty: String = "░".repeat(empty_len);

    format!("{}{}", filled, empty)
}

fn format_bytes_rate(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB/s", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB/s", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB/s", bytes as f64 / 1024.0)
    } else {
        format!("{} B/s", bytes)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

