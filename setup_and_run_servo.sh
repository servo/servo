#!/bin/bash
#
# Setup and Run Servo Browser
#
# This script sets up the development environment, performs a clean build of Servo,
# and launches it in GUI mode to connect to the UDS server.
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Configuration
SOCKET_DIR="/tmp/servo-sockets"
SOCKET_PATH="$SOCKET_DIR/localhost.sock"
URL="http::unix//localhost/"

echo -e "${CYAN}╔════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}  ${BOLD}Setup and Run Servo Browser${NC}                ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════╝${NC}"
echo

# Step 1: Check for UDS server
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 1: Checking for UDS Server${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

if [ ! -S "$SOCKET_PATH" ]; then
    echo -e "${RED}✗ UDS server not running${NC}"
    echo
    echo -e "${YELLOW}The UDS server must be running first!${NC}"
    echo
    echo -e "Start it in another terminal with:"
    echo -e "  ${GREEN}./setup_and_run_server.sh${NC}"
    echo
    exit 1
fi

echo -e "${GREEN}✓ UDS server is running${NC}"
echo -e "  Socket: ${CYAN}$SOCKET_PATH${NC}"
echo

# Step 2: Check Rust environment
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 2: Checking Rust Environment${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

if ! command -v rustc >/dev/null 2>&1; then
    echo -e "${RED}✗ Rust not found${NC}"
    echo
    echo -e "${YELLOW}Install Rust:${NC}"
    echo -e "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo
    exit 1
fi

echo -e "${GREEN}✓ Rust: $(rustc --version)${NC}"
echo -e "${GREEN}✓ Cargo: $(cargo --version)${NC}"
echo

# Step 3: Check system dependencies
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 3: Checking System Dependencies${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

missing_deps=()

# Check for essential build tools
if ! command -v python3 >/dev/null 2>&1; then
    missing_deps+=("python3")
fi

if ! command -v pkg-config >/dev/null 2>&1; then
    missing_deps+=("pkg-config")
fi

if [ ${#missing_deps[@]} -gt 0 ]; then
    echo -e "${RED}✗ Missing dependencies: ${missing_deps[*]}${NC}"
    echo
    echo -e "${YELLOW}Install with:${NC}"
    echo -e "  sudo apt install ${missing_deps[*]}"
    echo
    exit 1
fi

echo -e "${GREEN}✓ Essential build tools available${NC}"
echo

# Step 4: Clean build Servo
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 4: Clean Building Servo${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

echo -e "${YELLOW}This will perform a clean build. This may take 10-30 minutes...${NC}"
echo

# Clean previous builds
if [ -d "target" ]; then
    echo -e "${YELLOW}Cleaning previous build artifacts...${NC}"
    cargo clean
    echo -e "${GREEN}✓ Clean complete${NC}"
fi

# Build Servo in debug mode (faster to compile)
echo -e "${YELLOW}Building Servo (debug mode)...${NC}"
echo -e "${CYAN}(Build output will appear below)${NC}"
echo

if ./mach build -d; then
    echo
    echo -e "${GREEN}✓ Servo built successfully!${NC}"
else
    echo
    echo -e "${RED}✗ Servo build failed${NC}"
    exit 1
fi
echo

# Step 5: Verify Servo binary
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 5: Verifying Servo Binary${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

SERVO_BIN=""
if [ -f "target/debug/servo" ]; then
    SERVO_BIN="target/debug/servo"
    echo -e "${GREEN}✓ Found Servo: $SERVO_BIN${NC}"
elif [ -f "target/release/servo" ]; then
    SERVO_BIN="target/release/servo"
    echo -e "${GREEN}✓ Found Servo: $SERVO_BIN${NC}"
else
    echo -e "${RED}✗ Servo binary not found${NC}"
    exit 1
fi

# Get binary size
BINARY_SIZE=$(du -h "$SERVO_BIN" | cut -f1)
echo -e "  Binary size: ${CYAN}$BINARY_SIZE${NC}"
echo

# Step 6: Configure environment for UDS
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 6: Configuring UDS Environment${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

export RUST_LOG="${RUST_LOG:-net=debug}"

echo -e "${GREEN}✓ Environment configured${NC}"
echo -e "  RUST_LOG=${CYAN}$RUST_LOG${NC}"
echo

# Display launch information
echo -e "${CYAN}╔════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}  ${BOLD}READY TO LAUNCH SERVO${NC}                      ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════╝${NC}"
echo
echo -e "${BOLD}Configuration:${NC}"
echo -e "  UDS Mode:       ${GREEN}Enabled (default)${NC}"
echo -e "  Socket Dir:     ${CYAN}$SOCKET_DIR${NC}"
echo -e "  Socket Path:    ${CYAN}$SOCKET_PATH${NC}"
echo -e "  URL to Load:    ${GREEN}$URL${NC}"
echo -e "  Servo Binary:   ${CYAN}$SERVO_BIN${NC}"
echo
echo -e "${CYAN}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Navigation Tips:${NC}"
echo -e "${CYAN}════════════════════════════════════════════════${NC}"
echo
echo -e "  • Type URLs in the address bar using:"
echo -e "    ${GREEN}http::unix//localhost/path${NC}"
echo
echo -e "  • Example pages to try:"
echo -e "    ${BLUE}http::unix//localhost/${NC}         - Home page"
echo -e "    ${BLUE}http::unix//localhost/test${NC}     - Test page"
echo -e "    ${BLUE}http::unix//localhost/about${NC}    - About page"
echo -e "    ${BLUE}http::unix//localhost/api/data${NC} - JSON API"
echo
echo -e "  • Press ${BOLD}Ctrl+C${NC} to close the browser"
echo
echo -e "${CYAN}════════════════════════════════════════════════${NC}"
echo

# Step 7: Launch Servo
echo -e "${BOLD}${GREEN}🚀 Launching Servo...${NC}"
echo
echo -e "${YELLOW}Server logs will appear in the server terminal when you connect${NC}"
echo
echo -e "${CYAN}════════════════════════════════════════════════${NC}"
echo

# Launch Servo in GUI mode
"$SERVO_BIN" "$URL"

echo
echo -e "${GREEN}✓ Servo closed${NC}"
