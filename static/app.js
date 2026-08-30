const THEMES = [
    { id: 'theme-split', label: 'Diseño: Modular Split' },
    { id: 'theme-bento', label: 'Diseño: Bento Grid' },
    { id: 'theme-cyber', label: 'Diseño: Cyber-HUD' }
];

let currentThemeIndex = 0;

document.addEventListener('DOMContentLoaded', () => {
    // Initialize Theme
    initTheme();

    // Initial fetch
    fetchTelemetry();
    fetchSsdReport();
    fetchHistory();
    fetchThresholds();

    // Polling every 1000ms
    setInterval(fetchTelemetry, 1000);
    setInterval(fetchHistory, 5000);

    // Event listeners
    document.getElementById('btn-theme-switch').addEventListener('click', cycleTheme);
    document.getElementById('btn-manual-snapshot').addEventListener('click', saveManualSnapshot);
    document.getElementById('threshold-form').addEventListener('submit', saveThresholds);

    const inputCpu = document.getElementById('input-cpu-thresh');
    const inputRam = document.getElementById('input-ram-thresh');

    inputCpu.addEventListener('input', (e) => {
        document.getElementById('lbl-cpu-thresh').textContent = `${e.target.value}%`;
    });

    inputRam.addEventListener('input', (e) => {
        document.getElementById('lbl-ram-thresh').textContent = `${e.target.value}%`;
    });
});

function initTheme() {
    const saved = localStorage.getItem('matrix1_theme');
    if (saved) {
        const found = THEMES.findIndex(t => t.id === saved);
        if (found !== -1) currentThemeIndex = found;
    }
    applyTheme(THEMES[currentThemeIndex]);
}

function cycleTheme() {
    currentThemeIndex = (currentThemeIndex + 1) % THEMES.length;
    const theme = THEMES[currentThemeIndex];
    applyTheme(theme);
    localStorage.setItem('matrix1_theme', theme.id);
    showToast(`🎨 ${theme.label}`);
}

function applyTheme(theme) {
    document.body.className = theme.id;
    const lbl = document.getElementById('lbl-current-theme');
    if (lbl) lbl.textContent = theme.label;
}


async function fetchTelemetry() {
    try {
        const res = await fetch('/api/telemetry');
        if (!res.ok) return;
        const data = await res.json();
        updateDashboard(data);
    } catch (err) {
        console.error('Error fetching telemetry:', err);
    }
}

async function fetchSsdReport() {
    try {
        const res = await fetch('/api/ssd');
        if (!res.ok) return;
        const reports = await res.json();
        renderDisksList(reports);
    } catch (err) {
        console.error('Error fetching SSD report:', err);
    }
}

function renderDisksList(reports) {
    const container = document.getElementById('disks-container');
    const badgeCount = document.getElementById('disks-count-badge');

    badgeCount.textContent = `${reports.length} Disco(s) Detectado(s)`;

    if (reports.length === 0) {
        const emptyDiv = document.createElement('div');
        emptyDiv.className = 'text-center';
        emptyDiv.textContent = 'No se detectaron unidades de almacenamiento.';
        container.replaceChildren(emptyDiv);
        return;
    }

    const fragment = document.createDocumentFragment();
    reports.forEach(disk => {
        const card = document.createElement('div');
        card.className = 'disk-card-item';

        // Header
        const header = document.createElement('div');
        header.className = 'card-header';

        const headerLeft = document.createElement('div');
        headerLeft.style.display = 'flex';
        headerLeft.style.alignItems = 'center';
        headerLeft.style.gap = '8px';

        const typeBadgeClass = disk.is_hdd ? 'badge-hdd' : 'badge-ssd';
        const diskIcon = disk.is_hdd ? '⚙️ HDD' : (disk.is_nvme ? '⚡ NVMe' : '💽 SSD');
        const typeSpan = document.createElement('span');
        typeSpan.className = `disk-type-badge ${typeBadgeClass}`;
        typeSpan.textContent = `${diskIcon} ${disk.drive_type}`;

        const devSpan = document.createElement('span');
        devSpan.className = 'mono-sub';
        devSpan.textContent = `/dev/${disk.device_name}`;

        headerLeft.appendChild(typeSpan);
        headerLeft.appendChild(devSpan);

        const statusBadgeClass = disk.status === 'HEALTHY' ? 'badge-success' : 'badge-warning';
        const statusSpan = document.createElement('span');
        statusSpan.className = `badge ${statusBadgeClass}`;
        statusSpan.textContent = disk.status;

        header.appendChild(headerLeft);
        header.appendChild(statusSpan);

        // Model Box
        const modelBox = document.createElement('div');
        modelBox.className = 'ssd-model-box';

        const modelLabel = document.createElement('span');
        modelLabel.className = 'meta-label';
        modelLabel.textContent = 'DISCO / MODELO';

        const modelH3 = document.createElement('h3');
        modelH3.textContent = disk.model;

        const serialSpan = document.createElement('span');
        serialSpan.className = 'mono-sub';
        serialSpan.textContent = `S/N: ${disk.serial_number}`;

        modelBox.appendChild(modelLabel);
        modelBox.appendChild(modelH3);
        modelBox.appendChild(serialSpan);

        // EOL Box
        const eolBox = document.createElement('div');
        eolBox.className = 'eol-display-box';

        const eolLeft = document.createElement('div');
        eolLeft.className = 'eol-left';

        const eolLabel = document.createElement('span');
        eolLabel.className = 'meta-label';
        eolLabel.textContent = 'PROYECCIÓN FIN DE VIDA (EOL)';

        const eolDate = document.createElement('div');
        eolDate.className = 'eol-date-highlight';
        eolDate.textContent = disk.estimated_eol_date;

        const eolLife = document.createElement('span');
        eolLife.className = 'mono-sub';
        eolLife.style.color = 'var(--text-secondary)';
        eolLife.textContent = `Vida útil estimada: ${disk.estimated_lifetime_years.toFixed(1)} Años`;

        eolLeft.appendChild(eolLabel);
        eolLeft.appendChild(eolDate);
        eolLeft.appendChild(eolLife);

        const eolRight = document.createElement('div');
        eolRight.className = 'eol-right';

        const healthPct = document.createElement('span');
        healthPct.className = 'health-percentage';
        healthPct.style.color = disk.health_percentage > 70 ? 'var(--status-success)' : 'var(--status-warning)';
        healthPct.textContent = `${disk.health_percentage.toFixed(1)}%`;

        const healthLabel = document.createElement('span');
        healthLabel.className = 'meta-label';
        healthLabel.textContent = 'SALUD RESTANTE';

        eolRight.appendChild(healthPct);
        eolRight.appendChild(healthLabel);

        eolBox.appendChild(eolLeft);
        eolBox.appendChild(eolRight);

        // Metrics Grid
        const metricsGrid = document.createElement('div');
        metricsGrid.className = 'ssd-metrics-grid';

        const m1 = document.createElement('div');
        m1.className = 'ssd-metric';
        const m1Label = document.createElement('span');
        m1Label.className = 'meta-label';
        m1Label.textContent = 'TIEMPO ENCENDIDO (SMART)';
        const m1Val = document.createElement('span');
        m1Val.className = 'mono-val';
        m1Val.style.fontSize = '11px';
        m1Val.textContent = disk.power_on_formatted || (`${disk.power_on_hours} hrs`);
        m1.appendChild(m1Label);
        m1.appendChild(m1Val);

        const m2 = document.createElement('div');
        m2.className = 'ssd-metric';
        const m2Label = document.createElement('span');
        m2Label.className = 'meta-label';
        m2Label.textContent = 'CICLOS ENCENDIDO';
        const m2Val = document.createElement('span');
        m2Val.className = 'mono-val';
        m2Val.textContent = disk.power_cycle_count > 0 ? `${disk.power_cycle_count} ciclos` : 'N/D';
        m2.appendChild(m2Label);
        m2.appendChild(m2Val);

        const m3 = document.createElement('div');
        m3.className = 'ssd-metric';
        const m3Label = document.createElement('span');
        m3Label.className = 'meta-label';
        m3Label.textContent = 'ESCRITURA DIARIA PROM.';
        const m3Val = document.createElement('span');
        m3Val.className = 'mono-val';
        m3Val.textContent = disk.daily_write_gb ? `${disk.daily_write_gb.toFixed(1)} GB/día` : 'N/D';
        m3.appendChild(m3Label);
        m3.appendChild(m3Val);

        metricsGrid.appendChild(m1);
        metricsGrid.appendChild(m2);
        metricsGrid.appendChild(m3);

        // Recommendation
        const recBox = document.createElement('div');
        recBox.className = 'ssd-recommendation';
        const recIcon = document.createElement('span');
        recIcon.className = 'info-icon';
        recIcon.textContent = '💡';
        const recText = document.createElement('span');
        recText.textContent = disk.recommendation;
        recBox.appendChild(recIcon);
        recBox.appendChild(recText);

        card.appendChild(header);
        card.appendChild(modelBox);
        card.appendChild(eolBox);
        card.appendChild(metricsGrid);
        card.appendChild(recBox);

        fragment.appendChild(card);
    });

    container.replaceChildren(fragment);
}

async function fetchHistory() {
    try {
        const res = await fetch('/api/history');
        if (!res.ok) return;
        const list = await res.json();
        updateHistoryTable(list);
    } catch (err) {
        console.error('Error fetching history:', err);
    }
}

async function fetchThresholds() {
    try {
        const res = await fetch('/api/thresholds');
        if (!res.ok) return;
        const cfg = await res.json();

        document.getElementById('input-cpu-thresh').value = cfg.cpu_threshold;
        document.getElementById('lbl-cpu-thresh').textContent = `${cfg.cpu_threshold}%`;

        document.getElementById('input-ram-thresh').value = cfg.memory_threshold;
        document.getElementById('lbl-ram-thresh').textContent = `${cfg.memory_threshold}%`;

        document.getElementById('chk-auto-capture').checked = cfg.auto_capture_enabled;
    } catch (err) {
        console.error('Error fetching thresholds:', err);
    }
}

function updateDashboard(data) {
    // Meta System Info
    document.getElementById('sys-hostname').textContent = data.hostname;
    document.getElementById('sys-kernel').textContent = `${data.os_name} (${data.kernel_version})`;
    
    const uptimeMins = Math.floor(data.uptime_seconds / 60);
    const uptimeHours = Math.floor(uptimeMins / 60);
    document.getElementById('sys-uptime').textContent = `${uptimeHours}h ${uptimeMins % 60}m`;

    // Status Badge
    const statusBadge = document.getElementById('sys-status-badge');
    const statusText = document.getElementById('sys-status-text');
    statusText.textContent = data.status_level;

    statusBadge.className = 'status-badge';
    if (data.status_level === 'WARNING') statusBadge.classList.add('warning');
    if (data.status_level === 'CRITICAL') statusBadge.classList.add('critical');

    // CPU Gauge (Standard)
    const cpuPct = data.cpu_usage_total.toFixed(1);
    document.getElementById('cpu-pct').textContent = `${cpuPct}%`;
    document.getElementById('cpu-count-label').textContent = `${data.cpu_count} Cores`;
    updateGauge('cpu-gauge-circle', data.cpu_usage_total);

    // CPU Mini Cores
    const coresGrid = document.getElementById('cpu-cores-grid');
    const coresFragment = document.createDocumentFragment();
    data.cpu_cores.forEach(c => {
        const item = document.createElement('div');
        item.className = 'core-item';

        const label = document.createElement('span');
        label.textContent = `C${c.core_id}: ${c.usage.toFixed(0)}%`;

        const barBg = document.createElement('div');
        barBg.className = 'core-bar-bg';

        const barFill = document.createElement('div');
        barFill.className = 'core-bar-fill';
        barFill.style.width = `${c.usage}%`;

        barBg.appendChild(barFill);
        item.appendChild(label);
        item.appendChild(barBg);

        coresFragment.appendChild(item);
    });
    coresGrid.replaceChildren(coresFragment);

    // RAM Gauge (Standard)
    const ramPct = data.memory_percent.toFixed(1);
    document.getElementById('ram-pct').textContent = `${ramPct}%`;
    document.getElementById('ram-mb-label').textContent = `${data.memory_used_mb} / ${data.memory_total_mb} MB`;
    updateGauge('ram-gauge-circle', data.memory_percent);

    // ================= CYBER-HUD THEME 3 GAUGES =================
    // Cyber CPU HUD Arc
    const cyberCpuTop = document.getElementById('cyber-cpu-top-val');
    if (cyberCpuTop) cyberCpuTop.textContent = `${cpuPct}% USED / ${data.cpu_count} Cores`;
    const cyberCpuCenter = document.getElementById('cyber-cpu-center-val');
    if (cyberCpuCenter) cyberCpuCenter.textContent = `${cpuPct}%`;
    const cyberCpuLoad = document.getElementById('cyber-cpu-load-val');
    if (cyberCpuLoad) cyberCpuLoad.textContent = (data.cpu_usage_total / 100 * data.cpu_count).toFixed(2);
    updateGauge('cyber-cpu-arc', data.cpu_usage_total);

    // Cyber RAM/SWAP Concentric Dual Arc (Exactly matching concept art)
    const ramGbUsed = (data.memory_used_mb / 1024).toFixed(1);
    const ramGbTotal = (data.memory_total_mb / 1024).toFixed(0);
    const swapGbUsed = (data.swap_used_mb / 1024).toFixed(1);
    const swapGbTotal = (data.swap_total_mb / 1024).toFixed(0);

    const cyberRamTop = document.getElementById('cyber-ram-top-val');
    if (cyberRamTop) cyberRamTop.textContent = `${ramPct}% USED / ${ramGbTotal}GB Total`;

    const cyberRamCenterGb = document.getElementById('cyber-ram-center-gb');
    if (cyberRamCenterGb) cyberRamCenterGb.textContent = `${ramGbUsed} GB`;

    const cyberSwapCenterGb = document.getElementById('cyber-swap-center-gb');
    if (cyberSwapCenterGb) cyberSwapCenterGb.textContent = `${swapGbUsed} GB`;

    const cyberSwapBottom = document.getElementById('cyber-swap-bottom-val');
    if (cyberSwapBottom) cyberSwapBottom.textContent = `${data.swap_percent.toFixed(0)}% USED / ${swapGbTotal}GB Total`;

    updateGauge('cyber-ram-arc', data.memory_percent);
    updateGauge('cyber-swap-arc', data.swap_percent);

    // Bento Strip Metrics (Theme 2)
    const bentoCpuVal = document.getElementById('bento-cpu-val');
    if (bentoCpuVal) bentoCpuVal.textContent = `${cpuPct}%`;
    const bentoCpuBar = document.getElementById('bento-cpu-bar');
    if (bentoCpuBar) bentoCpuBar.style.width = `${data.cpu_usage_total}%`;

    const bentoRamVal = document.getElementById('bento-ram-val');
    if (bentoRamVal) bentoRamVal.textContent = `${data.memory_used_mb} / ${data.memory_total_mb} MB (${ramPct}%)`;
    const bentoRamBar = document.getElementById('bento-ram-bar');
    if (bentoRamBar) bentoRamBar.style.width = `${data.memory_percent}%`;

    const bentoSwapVal = document.getElementById('bento-swap-val');
    if (bentoSwapVal) bentoSwapVal.textContent = `${data.swap_used_mb} / ${data.swap_total_mb} MB (${data.swap_percent.toFixed(1)}%)`;
    const bentoSwapBar = document.getElementById('bento-swap-bar');
    if (bentoSwapBar) bentoSwapBar.style.width = `${data.swap_percent}%`;

    // Swap Bar
    document.getElementById('swap-val-label').textContent = `${data.swap_used_mb} / ${data.swap_total_mb} MB`;
    document.getElementById('swap-progress-fill').style.width = `${data.swap_percent}%`;

    // Disk Throughput Badges
    const readSpeedBadge = document.getElementById('disk-read-speed-badge');
    if (readSpeedBadge && data.total_disk_read_speed_mb !== undefined) {
        readSpeedBadge.textContent = `Lectura: ${data.total_disk_read_speed_mb.toFixed(1)} MB/s`;
    }
    const writeSpeedBadge = document.getElementById('disk-write-speed-badge');
    if (writeSpeedBadge && data.total_disk_write_speed_mb !== undefined) {
        writeSpeedBadge.textContent = `Escritura: ${data.total_disk_write_speed_mb.toFixed(1)} MB/s`;
    }

    // ================= TOP CPU PROCESSES =================
    // Standard Table
    const topCpuTbody = document.getElementById('top-cpu-tbody');
    const cpuFrag = document.createDocumentFragment();
    data.top_cpu_processes.forEach(p => {
        const tr = document.createElement('tr');
        const tdPid = document.createElement('td');
        tdPid.textContent = p.pid;
        const tdName = document.createElement('td');
        tdName.style.color = 'var(--accent-cyan)';
        tdName.textContent = p.name;
        const tdCpu = document.createElement('td');
        tdCpu.style.fontWeight = '700';
        tdCpu.textContent = `${p.cpu_usage.toFixed(1)}%`;
        const tdMem = document.createElement('td');
        tdMem.textContent = `${p.memory_mb} MB`;

        tr.appendChild(tdPid);
        tr.appendChild(tdName);
        tr.appendChild(tdCpu);
        tr.appendChild(tdMem);
        cpuFrag.appendChild(tr);
    });
    topCpuTbody.replaceChildren(cpuFrag);

    // Cyber HUD CPU Process List (Theme 3)
    const cyberCpuList = document.getElementById('cyber-top-cpu-list');
    if (cyberCpuList) {
        const cyberCpuFrag = document.createDocumentFragment();
        data.top_cpu_processes.forEach((p, idx) => {
            const item = document.createElement('div');
            item.className = 'cyber-proc-item';

            const row = document.createElement('div');
            row.className = 'cyber-proc-row';

            const left = document.createElement('div');
            left.className = 'cyber-proc-left';
            left.innerHTML = `<span class="cyber-proc-idx">${idx + 1}.</span><span class="cyber-proc-name">${p.name}</span>`;

            const right = document.createElement('div');
            right.className = 'cyber-proc-right';
            const memPill = p.memory_mb >= 1024 ? `${(p.memory_mb/1024).toFixed(1)} GB` : `${p.memory_mb} MB`;
            right.innerHTML = `<span class="cyber-pill cyber-pill-cyan">${memPill}</span><span class="cyber-proc-pct">${p.cpu_usage.toFixed(1)}%</span>`;

            row.appendChild(left);
            row.appendChild(right);

            const barBg = document.createElement('div');
            barBg.className = 'cyber-proc-bar-bg';
            const barFill = document.createElement('div');
            barFill.className = 'cyber-proc-bar-fill cyan-fill';
            barFill.style.width = `${Math.min(p.cpu_usage, 100)}%`;
            barBg.appendChild(barFill);

            item.appendChild(row);
            item.appendChild(barBg);
            cyberCpuFrag.appendChild(item);
        });
        cyberCpuList.replaceChildren(cyberCpuFrag);
    }

    // ================= TOP MEMORY PROCESSES =================
    // Standard Table
    const topRamTbody = document.getElementById('top-ram-tbody');
    const ramFrag = document.createDocumentFragment();
    data.top_memory_processes.forEach(p => {
        const tr = document.createElement('tr');
        const tdPid = document.createElement('td');
        tdPid.textContent = p.pid;
        const tdName = document.createElement('td');
        tdName.style.color = 'var(--accent-purple)';
        tdName.textContent = p.name;
        const tdMem = document.createElement('td');
        tdMem.style.fontWeight = '700';
        tdMem.textContent = `${p.memory_mb} MB`;
        const tdPct = document.createElement('td');
        tdPct.textContent = `${p.memory_percent.toFixed(1)}%`;

        tr.appendChild(tdPid);
        tr.appendChild(tdName);
        tr.appendChild(tdMem);
        tr.appendChild(tdPct);
        ramFrag.appendChild(tr);
    });
    topRamTbody.replaceChildren(ramFrag);

    // Cyber HUD RAM Process List (Theme 3 - Matches Concept Art)
    const cyberRamList = document.getElementById('cyber-top-ram-list');
    if (cyberRamList) {
        const cyberRamFrag = document.createDocumentFragment();
        data.top_memory_processes.forEach((p, idx) => {
            const item = document.createElement('div');
            item.className = 'cyber-proc-item';

            const row = document.createElement('div');
            row.className = 'cyber-proc-row';

            const left = document.createElement('div');
            left.className = 'cyber-proc-left';
            left.innerHTML = `<span class="cyber-proc-idx">${idx + 1}.</span><span class="cyber-proc-name">${p.name}</span>`;

            const right = document.createElement('div');
            right.className = 'cyber-proc-right';
            const memPill = p.memory_mb >= 1024 ? `${(p.memory_mb/1024).toFixed(1)} GB` : `${p.memory_mb} MB`;
            right.innerHTML = `<span class="cyber-pill cyber-pill-purple">${memPill}</span><span class="cyber-proc-pct">${p.memory_percent.toFixed(1)}%</span>`;

            row.appendChild(left);
            row.appendChild(right);

            const barBg = document.createElement('div');
            barBg.className = 'cyber-proc-bar-bg';
            const barFill = document.createElement('div');
            barFill.className = 'cyber-proc-bar-fill purple-fill';
            barFill.style.width = `${Math.min(p.memory_percent, 100)}%`;
            barBg.appendChild(barFill);

            item.appendChild(row);
            item.appendChild(barBg);
            cyberRamFrag.appendChild(item);
        });
        cyberRamList.replaceChildren(cyberRamFrag);
    }

    // Top Disk I/O Processes Table
    const topDiskTbody = document.getElementById('top-disk-tbody');
    if (topDiskTbody && data.top_disk_processes) {
        const diskFrag = document.createDocumentFragment();
        data.top_disk_processes.forEach(p => {
            const tr = document.createElement('tr');

            const tdPid = document.createElement('td');
            tdPid.textContent = p.pid;

            const tdName = document.createElement('td');
            tdName.style.color = 'var(--accent-blue)';
            tdName.textContent = p.name;

            const tdRead = document.createElement('td');
            tdRead.style.fontWeight = '600';
            tdRead.textContent = formatBytesRate(p.disk_read_bytes);

            const tdWrite = document.createElement('td');
            tdWrite.style.fontWeight = '600';
            tdWrite.textContent = formatBytesRate(p.disk_written_bytes);

            const tdTotal = document.createElement('td');
            tdTotal.textContent = formatBytes(p.disk_total_read_bytes + p.disk_total_written_bytes);

            tr.appendChild(tdPid);
            tr.appendChild(tdName);
            tr.appendChild(tdRead);
            tr.appendChild(tdWrite);
            tr.appendChild(tdTotal);

            diskFrag.appendChild(tr);
        });
        topDiskTbody.replaceChildren(diskFrag);
    }
}

function formatBytesRate(bytes) {
    if (!bytes || bytes === 0) return '0 B/s';
    if (bytes >= 1024 * 1024 * 1024) {
        return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB/s';
    } else if (bytes >= 1024 * 1024) {
        return (bytes / (1024 * 1024)).toFixed(1) + ' MB/s';
    } else if (bytes >= 1024) {
        return (bytes / 1024).toFixed(1) + ' KB/s';
    } else {
        return bytes + ' B/s';
    }
}

function formatBytes(bytes) {
    if (!bytes || bytes === 0) return '0 B';
    if (bytes >= 1024 * 1024 * 1024) {
        return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB';
    } else if (bytes >= 1024 * 1024) {
        return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
    } else if (bytes >= 1024) {
        return (bytes / 1024).toFixed(1) + ' KB';
    } else {
        return bytes + ' B';
    }
}

function updateHistoryTable(list) {
    const historyTbody = document.getElementById('history-tbody');
    if (list.length === 0) {
        const tr = document.createElement('tr');
        const td = document.createElement('td');
        td.colSpan = 7;
        td.className = 'text-center';
        td.textContent = 'No hay instantáneas guardadas aún.';
        tr.appendChild(td);
        historyTbody.replaceChildren(tr);
        return;
    }

    const fragment = document.createDocumentFragment();
    list.forEach(item => {
        const tr = document.createElement('tr');

        const tdId = document.createElement('td');
        tdId.textContent = `#${item.id}`;

        const tdTime = document.createElement('td');
        tdTime.textContent = item.timestamp;

        const tdTrigger = document.createElement('td');
        const badgeTrigger = document.createElement('span');
        badgeTrigger.className = 'badge';
        badgeTrigger.style.color = 'var(--accent-cyan)';
        badgeTrigger.textContent = item.trigger_type;
        tdTrigger.appendChild(badgeTrigger);

        const tdCpu = document.createElement('td');
        tdCpu.textContent = `${item.cpu_usage.toFixed(1)}%`;

        const tdRam = document.createElement('td');
        tdRam.textContent = `${item.memory_percent.toFixed(1)}%`;

        const tdProc = document.createElement('td');
        tdProc.textContent = item.top_process_name;

        const tdStatus = document.createElement('td');
        const badgeStatus = document.createElement('span');
        badgeStatus.className = 'badge';
        badgeStatus.textContent = item.status_level;
        tdStatus.appendChild(badgeStatus);

        tr.appendChild(tdId);
        tr.appendChild(tdTime);
        tr.appendChild(tdTrigger);
        tr.appendChild(tdCpu);
        tr.appendChild(tdRam);
        tr.appendChild(tdProc);
        tr.appendChild(tdStatus);

        fragment.appendChild(tr);
    });
    historyTbody.replaceChildren(fragment);
}

function updateGauge(elementId, percentage) {
    const circle = document.getElementById(elementId);
    if (!circle) return;
    const radius = circle.r.baseVal.value;
    const circumference = 2 * Math.PI * radius;
    const offset = circumference - (percentage / 100) * circumference;
    circle.style.strokeDasharray = `${circumference} ${circumference}`;
    circle.style.strokeDashoffset = offset;
}

async function saveManualSnapshot() {
    try {
        const res = await fetch('/api/snapshot', { method: 'POST' });
        if (res.ok) {
            showToast('📸 Instantánea guardada con éxito');
            fetchHistory();
        }
    } catch (err) {
        console.error('Error saving snapshot:', err);
    }
}

async function saveThresholds(e) {
    e.preventDefault();
    const cpuVal = parseFloat(document.getElementById('input-cpu-thresh').value);
    const ramVal = parseFloat(document.getElementById('input-ram-thresh').value);
    const autoCap = document.getElementById('chk-auto-capture').checked;

    const payload = {
        cpu_threshold: cpuVal,
        memory_threshold: ramVal,
        disk_io_threshold_mb: 100.0,
        auto_capture_enabled: autoCap
    };

    try {
        const res = await fetch('/api/thresholds', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });

        if (res.ok) {
            showToast('⚙️ Configuración de umbrales guardada');
        }
    } catch (err) {
        console.error('Error saving thresholds:', err);
    }
}

function showToast(msg) {
    const toast = document.getElementById('toast');
    toast.textContent = msg;
    toast.classList.remove('hidden');
    setTimeout(() => {
        toast.classList.add('hidden');
    }, 3000);
}
