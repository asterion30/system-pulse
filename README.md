# ⚡ Matrix1 SystemPulse & System-CLI-Pulse

**SystemPulse** es una suite de monitoreo de recursos del sistema en tiempo real y diagnóstico avanzado de salud y proyección de vida útil de unidades de almacenamiento (SSD/HDD) para sistemas operativos Linux (Debian, DietPi, Fedora, Arch, Ubuntu, RHEL y derivados).

El proyecto genera **dos binarios complementarios compilados en Rust**:
1. 🌐 **`system-pulse`**: Servidor Web Axum autónomo con interfaz gráfica Glassmorphic embebida (`rust-embed`) en el puerto 9090.
2. 💻 **`system-cli-pulse`**: Monitor interactivo de alto rendimiento para **consolas bash / terminales SSH en servidores sin entorno gráfico (headless como DietPi)**.

---

## 🌟 Binarios Disponibles

### 1. `system-cli-pulse` (Para Servidores Headless / DietPi sin GUI)
- **Tamaño Ultraligero**: Binario autónomo de solo **~3.8 MB** (cero dependencias de interfaz web o node).
- **Monitor en Tiempo Real Interactivo**: Dashboard visual ANSI / Unicode directo en tu terminal bash.
- **Vistas Especializadas**:
  - `system-cli-pulse` : Monitor interactivo en vivo con refresco continuo y captura con Ctrl+C.
  - `system-cli-pulse --summary` (`-s`) : Resumen estático para scripts, cron o bienvenidas SSH (MOTD).
  - `system-cli-pulse --disks` (`-d`) : Diagnóstico SMART detallado y proyección EOL de discos SSD/HDD.
  - `system-cli-pulse --json` (`-j`) : Exportación completa de métricas en JSON.
  - `system-cli-pulse --snapshot` : Guarda una captura manual inmediata en SQLite.

### 2. `system-pulse` (Servidor Web & API REST)
- Levanta el servidor HTTP en el puerto `9090`.
- Permite acceder desde cualquier navegador web en la red local (`http://<IP-HOST>:9090`).
- Ideal para monitorear servidores remotamente desde tu PC, laptop o teléfono.

---

## 📦 Compilación

Para compilar ambos binarios en modo Release optimizado:

```bash
cargo build --release
```

Los binarios generados se encontrarán en:
- `target/release/system-cli-pulse` (Binario de consola)
- `target/release/system-pulse` (Binario servidor web)

---

## 🚀 Trasladar y Ejecutar `system-cli-pulse` a una máquina DietPi

Para llevar el binario de consola a tu servidor DietPi (u otro Linux sin entorno de escritorio):

```bash
# 1. Copiar el binario por SSH/SCP a tu DietPi
scp target/release/system-cli-pulse root@<IP_DIETPI>:/usr/local/bin/

# 2. Conectarte por SSH a tu DietPi
ssh root@<IP_DIETPI>

# 3. Dar permisos de ejecución y correr
system-cli-pulse
```

---

## 🏗️ Arquitectura del Proyecto

```
system-pulse/
├── Cargo.toml               # Configuración multi-binario (system-pulse y system-cli-pulse)
├── src/
│   ├── lib.rs               # Biblioteca base compartida
│   ├── main.rs              # Punto de entrada para el servidor Web (system-pulse)
│   ├── cli_main.rs          # Punto de entrada para la consola Bash (system-cli-pulse)
│   ├── web_server.rs        # Servidor HTTP Axum & enrutador REST
│   ├── telemetry.rs         # Colector de métricas de sistema (CPU, RAM, Procesos)
│   ├── ssd_analyzer.rs      # Diagnóstico SMART UDisks2 / sysfs & motor probabilístico EOL
│   ├── db.rs                # Gestor de base de datos SQLite y persistencia de eventos
│   └── threshold_engine.rs  # Motor de monitoreo automático de umbrales
└── static/
    ├── index.html           # Dashboard principal
    ├── style.css            # Estilos CSS
    └── app.js               # Lógica cliente JS (Seguro DOM / SAST 100/100)
```

---

## 📜 Licencia

Distribuido bajo la Licencia MIT.
