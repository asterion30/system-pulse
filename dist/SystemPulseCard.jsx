import React, { useState } from 'react';

export default function SystemPulseCard() {
    const [copied, setCopied] = useState(false);
    const installCmd = 'curl -fsSL https://raw.githubusercontent.com/asterion30/system-pulse/main/install.sh | bash';

    const handleCopy = () => {
        navigator.clipboard.writeText(installCmd);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <div className="relative group p-6 rounded-xl border border-cyan-500/30 bg-slate-950/80 backdrop-blur-md shadow-2xl hover:border-cyan-400 transition-all duration-300">
            {/* Top Bar */}
            <div className="flex justify-between items-center mb-3">
                <div className="w-10 h-10 rounded-lg bg-cyan-500/10 border border-cyan-500/30 flex items-center justify-center text-xl text-cyan-400">
                    ⚡
                </div>
                <span className="px-2.5 py-1 text-[10px] font-mono font-bold tracking-wider text-yellow-400 border border-yellow-500/40 bg-yellow-500/10 rounded">
                    SYS TELEMETRY & IA LAB
                </span>
            </div>

            {/* Meta ID & Title */}
            <div className="font-mono text-xs text-slate-500 mb-1">[ID // PRJ-05_SYSPULSE]</div>
            <h3 className="text-xl font-bold text-white mb-2 tracking-wide group-hover:text-cyan-300 transition-colors">
                System Pulse
            </h3>

            {/* Description */}
            <p className="text-xs text-slate-400 leading-relaxed mb-4">
                Monitor de telemetría de alto rendimiento en tiempo real (CPU, RAM, I/O diferencial) y motor predictivo de fin de vida útil SSD/HDD (SMART/EOL). Empaquetado estático sin dependencias para Debian, Fedora, Arch y DietPi.
            </p>

            {/* Security Badges */}
            <div className="flex flex-wrap gap-2 mb-4">
                <span className="px-2 py-0.5 text-[10px] font-mono font-semibold text-emerald-400 bg-emerald-500/10 border border-emerald-500/30 rounded">
                    🔒 Rust Musl Static
                </span>
                <span className="px-2 py-0.5 text-[10px] font-mono font-semibold text-emerald-400 bg-emerald-500/10 border border-emerald-500/30 rounded">
                    🛡️ SHA-256 Verificado
                </span>
                <span className="px-2 py-0.5 text-[10px] font-mono font-semibold text-cyan-400 bg-cyan-500/10 border border-cyan-500/30 rounded">
                    ⚡ Zero Dependencies
                </span>
            </div>

            {/* Tags */}
            <div className="flex gap-2 font-mono text-[11px] text-cyan-400/80 mb-4">
                <span>#RUST</span>
                <span>#LINUX</span>
                <span>#SMART_EOL</span>
                <span>#OPEN_SOURCE</span>
            </div>

            {/* Fast Install Snippet */}
            <div className="bg-black/60 border border-dashed border-cyan-500/30 rounded-lg p-3 mb-4">
                <div className="flex justify-between items-center text-[10px] font-bold text-slate-400 mb-1">
                    <span>INSTALACIÓN RÁPIDA (CLI & WEB):</span>
                    <button
                        onClick={handleCopy}
                        className="px-2 py-0.5 rounded bg-cyan-500/15 hover:bg-cyan-500 hover:text-black border border-cyan-500/40 text-cyan-400 text-[10px] transition-all font-mono"
                    >
                        {copied ? '✓ Copiado' : '📋 Copiar'}
                    </button>
                </div>
                <code className="block font-mono text-[11px] text-sky-300 break-all select-all">
                    {installCmd}
                </code>
            </div>

            {/* Actions */}
            <div className="grid grid-cols-2 gap-3">
                <a
                    href="https://github.com/asterion30/system-pulse/releases/latest"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center justify-center py-2 px-3 text-[11px] font-bold tracking-wider text-cyan-400 border border-cyan-400/80 rounded hover:bg-cyan-400 hover:text-black hover:shadow-[0_0_15px_rgba(0,242,254,0.4)] transition-all text-center"
                >
                    📥 DESCARGAR (.tar.gz)
                </a>
                <a
                    href="https://github.com/asterion30/system-pulse"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex items-center justify-center py-2 px-3 text-[11px] font-bold tracking-wider text-slate-400 border border-slate-700 rounded hover:border-slate-400 hover:text-white transition-all text-center"
                >
                    🐙 GITHUB
                </a>
            </div>
        </div>
    );
}
