#!/usr/bin/env bash
# ==============================================================================
# ⚡ Matrix1 SystemPulse - Script de Instalación Segura y Verificada
# Repositorio: https://github.com/asterion30/system-pulse
# ==============================================================================

set -euo pipefail

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
RESET='\033[0m'

REPO="asterion30/system-pulse"

echo -e "${CYAN}${BOLD}"
echo "  ⚡ ====================================================== ⚡"
echo "        MATRIX1 SYSTEM-PULSE - INSTALADOR SEGURO LINUX"
echo "  ⚡ ====================================================== ⚡"
echo -e "${RESET}"

# 1. Comprobar que el sistema operativo es Linux
OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
    echo -e "${RED}[ERROR] Este software está diseñado exclusivamente para entornos Linux.${RESET}"
    exit 1
fi

# 2. Detectar Arquitectura
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="linux-x86_64"
        ;;
    aarch64|arm64)
        TARGET_ARCH="linux-aarch64"
        ;;
    *)
        echo -e "${RED}[ERROR] Arquitectura no soportada: $ARCH (Soportadas: x86_64, aarch64)${RESET}"
        exit 1
        ;;
esac

echo -e " • Arquitectura detectada: ${GREEN}${TARGET_ARCH}${RESET}"

# 3. Obtener la última versión desde GitHub Releases
echo -e " • Consultando última versión oficial en GitHub..."
LATEST_TAG=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)

if [ -z "$LATEST_TAG" ]; then
    echo -e "${YELLOW}[AVISO] No se pudo obtener la última release mediante la API, usando 'main'...${RESET}"
    LATEST_TAG="latest"
fi

echo -e " • Versión a instalar: ${CYAN}${LATEST_TAG}${RESET}"

TAR_NAME="system-pulse-${LATEST_TAG}-${TARGET_ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${TAR_NAME}"
CHECKSUM_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/SHA256SUMS.txt"

# Crear directorio temporal seguro
TMP_DIR=$(mktemp -d /tmp/system-pulse-install.XXXXXX)
trap 'rm -rf "$TMP_DIR"' EXIT

cd "$TMP_DIR"

# 4. Descarga de artefactos
echo -e " • Descargando binario y sumas de verificación criptográficas (SHA-256)..."
if ! curl -fsSL "$DOWNLOAD_URL" -o "$TAR_NAME"; then
    echo -e "${RED}[ERROR] No se pudo descargar el paquete desde $DOWNLOAD_URL${RESET}"
    exit 1
fi

if ! curl -fsSL "$CHECKSUM_URL" -o "SHA256SUMS.txt"; then
    echo -e "${YELLOW}[AVISO] Archivo de sumas no disponible en la release.${RESET}"
else
    # 5. Verificación Criptográfica Estricta
    echo -e " • ${BOLD}Verificando integridad criptográfica SHA-256...${RESET}"
    if grep "$TAR_NAME" SHA256SUMS.txt | sha256sum --check --status 2>/dev/null; then
        echo -e "   ${GREEN}✓ Verificación SHA-256 exitosa: El binario es auténtico e íntegro.${RESET}"
    else
        echo -e "   ${RED}[ERROR FATAL] La suma de verificación SHA-256 no coincide.${RESET}"
        echo -e "   ${RED}El archivo descargado podría estar corrupto o comprometido. Abortando instalación.${RESET}"
        exit 1
    fi
fi

# 6. Extracción
echo -e " • Desempaquetando archivos..."
tar -xzf "$TAR_NAME"

# 7. Determinar ruta de instalación
if [ "$(id -u)" -eq 0 ]; then
    INSTALL_DIR="/usr/local/bin"
else
    INSTALL_DIR="${HOME}/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

echo -e " • Instalando binarios en ${GREEN}${INSTALL_DIR}${RESET}..."

# Localizar los ejecutables extraídos
EXTRACTED_DIR=$(find . -maxdepth 1 -type d -name "system-pulse-*" | head -n 1)
if [ -z "$EXTRACTED_DIR" ]; then
    EXTRACTED_DIR="."
fi

cp "${EXTRACTED_DIR}/system-pulse" "$INSTALL_DIR/"
cp "${EXTRACTED_DIR}/system-cli-pulse" "$INSTALL_DIR/"
chmod 755 "${INSTALL_DIR}/system-pulse"
chmod 755 "${INSTALL_DIR}/system-cli-pulse"

echo -e "${GREEN}${BOLD}"
echo "  🎉 ====================================================== 🎉"
echo "        ¡INSTALACIÓN COMPLETADA SATISFACTORIAMENTE!"
echo "  🎉 ====================================================== 🎉"
echo -e "${RESET}"

echo -e "Ejecutables instalados:"
echo -e "  • ${CYAN}system-pulse${RESET}       (Servidor Web Dashboard: http://127.0.0.1:9090)"
echo -e "  • ${CYAN}system-cli-pulse${RESET}   (Monitor Terminal TUI interactivo)"
echo ""

# Comprobar si INSTALL_DIR está en el PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}[NOTA] Añade $INSTALL_DIR a tu PATH agregando la siguiente línea a tu ~/.bashrc o ~/.zshrc:${RESET}"
    echo -e "  export PATH=\"\$PATH:$INSTALL_DIR\""
    echo ""
fi

echo -e "Para iniciar el monitor en consola ahora mismo, ejecuta:"
echo -e "  ${BOLD}system-cli-pulse${RESET}"
echo ""
echo -e "Para iniciar el panel web:"
echo -e "  ${BOLD}system-pulse${RESET}"
