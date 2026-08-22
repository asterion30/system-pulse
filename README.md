# ⚡ Matrix1 SystemPulse

**SystemPulse** es una aplicación de monitoreo de recursos del sistema en tiempo real y diagnóstico avanzado de salud y proyección de vida útil de unidades de almacenamiento (SSD/HDD) para sistemas operativos Linux (Debian, Fedora, Arch, Ubuntu, RHEL y derivados).

Construida con **Rust**, **Axum**, **SQLite** y una interfaz moderna **Glassmorphic Dark UI**.

---

## 🌟 Características Principales

- 🚀 **Telemetría en Tiempo Real**: Visualización dinámica de uso de CPU (total y multinúcleo), Memoria RAM, SWAP e I/O de disco.
- 🔥 **Top 5 Procesos Consumidores**: Ranking en vivo de las 5 aplicaciones que más recursos consumen por CPU y RAM.
- 💽 **Diagnóstico SMART Auténtico (UDisks2 / D-Bus)**: Integración directa con el demonio `org.freedesktop.UDisks2` (el mismo backend utilizado por Discos de GNOME / `gnome-disk-utility`).
  - Identificación exacta del modelo, número de serie único y tipo de unidad (SSD SATA, NVMe o HDD Mecánico).
  - Tiempo de encendido real de hardware (`SmartPowerOnSeconds` / SMART ID 9) en formato legible (años, meses, días).
  - Contador real de ciclos de encendido (SMART ID 12).
  - Temperatura en tiempo real del firmware del disco.
  - Tasa de escritura acumulada (TBW) y promedio de escritura diaria (GB/día).
- 🔮 **Proyección Matemáticamente Precisa de Fin de Vida (EOL)**: Algoritmo de estimación híbrido que combina la resistencia nominal NAND (TBW), el consumo diario de escritura y la curva de fatiga térmica/electrónica de componentes (máx. 10 años / 87,600 hrs).
- 🔔 **Motor de Umbrales & Captura Automática**: Disparador inteligente que guarda instantáneas de uso en SQLite cuando la CPU supera el 90% o la RAM el 85%.
- 📦 **Binario Autónomo y Portable**: Un único binario estático que incluye el servidor Axum y los activos web embebidos (`rust-embed`). No requiere instalar Node.js, Python ni servidores externos.

---

## 🛠️ Requisitos Previos

- **Linux** (Debian 12+, Ubuntu 22.04+, Fedora 38+, Arch Linux, RHEL, etc.).
- **Servicio `udisks2`** (instalado por defecto en la mayoría de distribuciones con entorno de escritorio GNOME/KDE).
- **Rust toolchain** (solo necesario para compilar desde el código fuente):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

---

## 📦 Compilación y Ejecución

### 1. Clonar el repositorio
```bash
git clone https://github.com/TU_USUARIO/system-pulse.git
cd system-pulse
```

### 2. Compilar en modo Release
```bash
cargo build --release
```

### 3. Ejecutar la aplicación
```bash
./target/release/system-pulse
```

Abre tu navegador en: **`http://localhost:9090`**

---

## 🏗️ Arquitectura del Proyecto

```
system-pulse/
├── Cargo.toml               # Configuración de dependencias Rust (Axum, sysinfo, rusqlite, rust-embed)
├── src/
│   ├── main.rs              # Punto de entrada de la aplicación
│   ├── web_server.rs        # Servidor HTTP Axum & enrutador de API REST
│   ├── telemetry.rs         # Colector de métricas de sistema (CPU, RAM, Procesos)
│   ├── ssd_analyzer.rs      # Diagnóstico SMART UDisks2 & motor probabilístico EOL
│   ├── db.rs                # Gestor de base de datos SQLite y persistencia de eventos
│   └── threshold_engine.rs  # Motor de monitoreo automático de umbrales
└── static/
    ├── index.html           # Dashboard principal (HTML5 semántico)
    ├── style.css            # Estilos Glassmorphic en CSS vanilla con variables de diseño
    └── app.js               # Renderizado dinámico cliente en JavaScript JS (ES6+)
```

---

## 📜 Licencia

Distribuido bajo la Licencia MIT.
