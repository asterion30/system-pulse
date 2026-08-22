document.addEventListener('DOMContentLoaded', () => {
    // Initial fetch
    fetchTelemetry();
    fetchSsdReport();
    fetchHistory();
    fetchThresholds();

    // Polling every 1000ms
    setInterval(fetchTelemetry, 1000);
    setInterval(fetchHistory, 5000);

    // Event listeners
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
        container.innerHTML = '<div class="text-center">No se detectaron unidades de almacenamiento.</div>';
        return;
    }

    container.innerHTML = reports.map(disk => {
        const typeBadgeClass = disk.is_hdd ? 'badge-hdd' : 'badge-ssd';
        const statusBadgeClass = disk.status === 'HEALTHY' ? 'badge-success' : 'badge-warning';
        const diskIcon = disk.is_hdd ? '⚙️ HDD' : (disk.is_nvme ? '⚡ NVMe' : '💽 SSD');
        const tempFahrenheit = (disk.temperature_celsius * 9 / 5) + 32;

        return `
            <div class="disk-card-item">
                <div class="card-header">
                    <div style="display:flex; align-items:center; gap:8px;">
                        <span class="disk-type-badge ${typeBadgeClass}">${diskIcon} ${disk.drive_type}</span>
                        <span class="mono-sub">/dev/${disk.device_name}</span>
                    </div>
                    <span class="badge ${statusBadgeClass}">${disk.status}</span>
                </div>

                <div class="ssd-model-box">
                    <span class="meta-label">DISCO / MODELO</span>
                    <h3>${escapeHtml(disk.model)}</h3>
                    <span class="mono-sub">S/N: ${escapeHtml(disk.serial_number)}</span>
                </div>

                <div class="eol-display-box">
                    <div class="eol-left">
                        <span class="meta-label">PROYECCIÓN FIN DE VIDA (EOL)</span>
                        <div class="eol-date-highlight">${disk.estimated_eol_date}</div>
                        <span class="mono-sub" style="color:var(--text-secondary);">Vida útil estimada: ${disk.estimated_lifetime_years.toFixed(1)} Años</span>
                    </div>
                    <div class="eol-right">
                        <span class="health-percentage" style="color: ${disk.health_percentage > 70 ? 'var(--status-success)' : 'var(--status-warning)'};">
                            ${disk.health_percentage.toFixed(1)}%
                        </span>
                        <span class="meta-label">SALUD RESTANTE</span>
                    </div>
                </div>

                <div class="ssd-metrics-grid">
                    <div class="ssd-metric">
                        <span class="meta-label">TIEMPO ENCENDIDO (SMART)</span>
                        <span class="mono-val" style="font-size:11px;">${disk.power_on_formatted || (disk.power_on_hours + ' hrs')}</span>
                    </div>
                    <div class="ssd-metric">
                        <span class="meta-label">CICLOS ENCENDIDO</span>
                        <span class="mono-val">${disk.power_cycle_count > 0 ? disk.power_cycle_count + ' ciclos' : 'N/D'}</span>
                    </div>
                    <div class="ssd-metric">
                        <span class="meta-label">ESCRITURA DIARIA PROM.</span>
                        <span class="mono-val">${disk.daily_write_gb ? disk.daily_write_gb.toFixed(1) + ' GB/día' : 'N/D'}</span>
                    </div>
                </div>

                <div class="ssd-recommendation">
                    <span class="info-icon">💡</span>
                    <span>${escapeHtml(disk.recommendation)}</span>
                </div>
            </div>
        `;
    }).join('');
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

    // CPU Gauge
    const cpuPct = data.cpu_usage_total.toFixed(1);
    document.getElementById('cpu-pct').textContent = `${cpuPct}%`;
    document.getElementById('cpu-count-label').textContent = `${data.cpu_count} Cores`;
    updateGauge('cpu-gauge-circle', data.cpu_usage_total);

    // CPU Mini Cores
    const coresGrid = document.getElementById('cpu-cores-grid');
    coresGrid.innerHTML = data.cpu_cores.map(c => `
        <div class="core-item">
            <span>C${c.core_id}: ${c.usage.toFixed(0)}%</span>
            <div class="core-bar-bg">
                <div class="core-bar-fill" style="width: ${c.usage}%"></div>
            </div>
        </div>
    `).join('');

    // RAM Gauge
    const ramPct = data.memory_percent.toFixed(1);
    document.getElementById('ram-pct').textContent = `${ramPct}%`;
    document.getElementById('ram-mb-label').textContent = `${data.memory_used_mb} / ${data.memory_total_mb} MB`;
    updateGauge('ram-gauge-circle', data.memory_percent);

    // Swap Bar
    document.getElementById('swap-val-label').textContent = `${data.swap_used_mb} / ${data.swap_total_mb} MB`;
    document.getElementById('swap-progress-fill').style.width = `${data.swap_percent}%`;

    // Top CPU Processes Table
    const topCpuTbody = document.getElementById('top-cpu-tbody');
    topCpuTbody.innerHTML = data.top_cpu_processes.map(p => `
        <tr>
            <td>${p.pid}</td>
            <td style="color: var(--accent-cyan);">${escapeHtml(p.name)}</td>
            <td style="font-weight:700;">${p.cpu_usage.toFixed(1)}%</td>
            <td>${p.memory_mb} MB</td>
        </tr>
    `).join('');

    // Top Memory Processes Table
    const topRamTbody = document.getElementById('top-ram-tbody');
    topRamTbody.innerHTML = data.top_memory_processes.map(p => `
        <tr>
            <td>${p.pid}</td>
            <td style="color: var(--accent-purple);">${escapeHtml(p.name)}</td>
            <td style="font-weight:700;">${p.memory_mb} MB</td>
            <td>${p.memory_percent.toFixed(1)}%</td>
        </tr>
    `).join('');
}



function updateHistoryTable(list) {
    const historyTbody = document.getElementById('history-tbody');
    if (list.length === 0) {
        historyTbody.innerHTML = '<tr><td colspan="7" class="text-center">No hay instantáneas guardadas aún.</td></tr>';
        return;
    }

    historyTbody.innerHTML = list.map(item => `
        <tr>
            <td>#${item.id}</td>
            <td>${item.timestamp}</td>
            <td><span class="badge" style="color:var(--accent-cyan);">${escapeHtml(item.trigger_type)}</span></td>
            <td>${item.cpu_usage.toFixed(1)}%</td>
            <td>${item.memory_percent.toFixed(1)}%</td>
            <td>${escapeHtml(item.top_process_name)}</td>
            <td><span class="badge">${item.status_level}</span></td>
        </tr>
    `).join('');
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

function escapeHtml(str) {
    return str.replace(/[&<>"']/g, m => ({
        '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#039;'
    })[m]);
}
