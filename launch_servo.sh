#!/bin/bash
#
# Launch Servo Browser
#
# Launches the Servo browser in GUI mode, configured to use Unix Domain Sockets.
# Make sure the UDS server is running first (run ./start_uds_server.sh).
#
# Usage:
#   ./launch_servo.sh                        # Launch with default URL
#   ./launch_servo.sh http://localhost/test  # Launch with custom URL
#   ./launch_servo.sh --headless             # Headless mode (screenshot)
#   ./launch_servo.sh --help                 # Show help
#

set -e

# Configuration
SOCKET_DIR="${SERVO_SOCKET_DIR:-/tmp/servo-sockets}"
SOCKET_PATH="$SOCKET_DIR/localhost.sock"
DEFAULT_URL="http::unix//localhost/"
MODE="gui"
URL=""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Help message
show_help() {
    echo -e "${CYAN}╔════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC}  ${BOLD}Launch Servo Browser (UDS Mode)${NC}            ${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════╝${NC}"
    echo
    echo "Launches Servo browser configured for Unix Domain Socket networking."
    echo
    echo -e "${BOLD}Usage:${NC}"
    echo "  $0 [URL] [OPTIONS]"
    echo
    echo -e "${BOLD}Arguments:${NC}"
    echo "  URL           Unix socket URL to load (default: http::unix//localhost/)"
    echo
    echo -e "${BOLD}Options:${NC}"
    echo "  --headless    Run in headless mode (screenshot)"
    echo "  --help        Show this help message"
    echo
    echo -e "${BOLD}Environment Variables:${NC}"
    echo "  SERVO_SOCKET_DIR   Socket directory (default: /tmp/servo-sockets)"
    echo
    echo -e "${BOLD}Prerequisites:${NC}"
    echo "  The UDS server must be running first!"
    echo "  Start it with: ${GREEN}./start_uds_server.sh${NC}"
    echo
    echo -e "${BOLD}URL Syntax:${NC}"
    echo "  ${CYAN}http::unix//localhost/path${NC}        Use hostname mapping"
    echo "  ${CYAN}http::unix///tmp/path.sock/page${NC}   Use explicit socket path"
    echo
    echo -e "${BOLD}Examples:${NC}"
    echo "  ./launch_servo.sh                              # Load http::unix//localhost/"
    echo "  ./launch_servo.sh http::unix//localhost/test   # Load /test page"
    echo "  ./launch_servo.sh http::unix//localhost/about  # Load /about page"
    echo "  ./launch_servo.sh --headless                   # Take screenshot"
    echo
    exit 0
}

# Parse arguments
for arg in "$@"; do
    case $arg in
        --help|-h)
            show_help
            ;;
        --headless)
            MODE="headless"
            ;;
        http://*)
            URL="$arg"
            ;;
        *)
            if [ -z "$URL" ]; then
                URL="$arg"
            fi
            ;;
    esac
done

# Set default URL if not provided
if [ -z "$URL" ]; then
    URL="$DEFAULT_URL"
fi

# Check if socket exists
check_socket() {
    echo -e "${BLUE}🔍 Checking for UDS server...${NC}"

    if [ ! -S "$SOCKET_PATH" ]; then
        echo -e "${RED}  ✗ Socket not found: $SOCKET_PATH${NC}"
        echo
        echo -e "${YELLOW}The UDS server is not running!${NC}"
        echo
        echo -e "Start the server first with:"
        echo -e "  ${GREEN}./start_uds_server.sh${NC}"
        echo
        exit 1
    fi

    echo -e "${GREEN}  ✓ UDS server is running${NC}"
    echo -e "    Socket: ${CYAN}$SOCKET_PATH${NC}"
    echo
}

# Find Servo binary
find_servo() {
    echo -e "${BLUE}🔍 Looking for Servo binary...${NC}" >&2

    local servo_bin=""
    if [ -f "target/release/servo" ]; then
        servo_bin="target/release/servo"
    elif [ -f "target/debug/servo" ]; then
        servo_bin="target/debug/servo"
    fi

    if [ -z "$servo_bin" ]; then
        echo -e "${RED}  ✗ Servo binary not found${NC}" >&2
        echo >&2
        echo -e "${YELLOW}Build Servo first:${NC}" >&2
        echo -e "  ${CYAN}./mach build${NC}         # Release build (recommended)" >&2
        echo -e "  ${CYAN}./mach build -d${NC}      # Debug build (faster compilation)" >&2
        echo >&2
        exit 1
    fi

    echo -e "${GREEN}  ✓ Found Servo: $servo_bin${NC}" >&2
    echo >&2

    echo "$servo_bin"
}

# Launch Servo
launch_servo() {
    local servo_bin="$1"

    # UDS mode is now enabled by default in Servo!
    # These environment variables are optional overrides
    export RUST_LOG="${RUST_LOG:-net=debug}"

    echo -e "${CYAN}════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}Servo Configuration:${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════${NC}"
    echo -e "  UDS Mode:          ${GREEN}Enabled (default)${NC}"
    echo -e "  Socket directory:  ${YELLOW}$SOCKET_DIR${NC}"
    echo -e "  Socket path:       ${YELLOW}$SOCKET_PATH${NC}"
    echo -e "  URL to load:       ${YELLOW}$URL${NC}"
    echo -e "  Servo binary:      ${YELLOW}$servo_bin${NC}"

    if [ "$MODE" = "headless" ]; then
        echo -e "  Mode:              ${YELLOW}Headless (screenshot)${NC}"
        echo -e "${CYAN}════════════════════════════════════════════════${NC}"
        echo
        echo -e "${GREEN}📸 Taking screenshot...${NC}"

        local screenshot="/tmp/servo-screenshot-$(date +%s).png"
        "$servo_bin" -z -x -o "$screenshot" "$URL"

        if [ -f "$screenshot" ]; then
            echo
            echo -e "${GREEN}✓ Screenshot saved: $screenshot${NC}"
            echo -e "  View with: ${CYAN}xdg-open $screenshot${NC}"
        else
            echo -e "${RED}✗ Screenshot failed${NC}"
        fi
    else
        echo -e "  Mode:              ${YELLOW}Interactive GUI${NC}"
        echo -e "${CYAN}════════════════════════════════════════════════${NC}"
        echo
        echo -e "${GREEN}🌐 Launching Servo browser...${NC}"
        echo
        echo -e "${YELLOW}Navigation tips:${NC}"
        echo -e "  • Use Unix socket URL syntax in the address bar:"
        echo -e "    ${GREEN}http::unix//localhost/test${NC}"
        echo -e "  • Try these pages: ${GREEN}/test${NC}, ${GREEN}/about${NC}, ${GREEN}/api/data${NC}"
        echo -e "  • Press ${BOLD}Ctrl+C${NC} to close the browser"
        echo
        echo -e "${CYAN}════════════════════════════════════════════════${NC}"
        echo

        # Launch Servo in GUI mode
        "$servo_bin" "$URL"
    fi
}

# Main function
main() {
    echo -e "${CYAN}╔════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC}  ${BOLD}Servo Browser Launcher (UDS Mode)${NC}          ${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════╝${NC}"
    echo

    check_socket
    SERVO_BIN=$(find_servo)
    launch_servo "$SERVO_BIN"

    echo
    echo -e "${GREEN}✓ Done${NC}"
}

# Run
main
