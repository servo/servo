#!/bin/bash
#
# Setup and Run UDS Server
#
# This script sets up the development environment and runs the Gunicorn server.
# It ensures all dependencies are installed and the server is running cleanly.
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
PID_FILE="/tmp/servo-uds-server.pid"

echo -e "${CYAN}╔════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}  ${BOLD}Setup and Run UDS Server${NC}                   ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════╝${NC}"
echo

# Cleanup function
cleanup() {
    echo
    echo -e "${YELLOW}🛑 Shutting down server...${NC}"

    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE" 2>/dev/null)
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
            wait "$pid" 2>/dev/null || true
        fi
        rm -f "$PID_FILE"
    fi

    rm -f "$SOCKET_PATH"
    echo -e "${GREEN}✓ Server stopped${NC}"
}

trap cleanup EXIT INT TERM

# Step 1: Check and setup Python environment
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 1: Checking Python Environment${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

if ! command -v python3 >/dev/null 2>&1; then
    echo -e "${RED}✗ Python3 not found${NC}"
    echo -e "${YELLOW}Install Python3:${NC}"
    echo -e "  sudo apt install python3 python3-pip python3-venv"
    exit 1
fi

echo -e "${GREEN}✓ Python3: $(python3 --version)${NC}"

if ! command -v pip3 >/dev/null 2>&1; then
    echo -e "${RED}✗ pip3 not found${NC}"
    echo -e "${YELLOW}Install pip3:${NC}"
    echo -e "  sudo apt install python3-pip"
    exit 1
fi

echo -e "${GREEN}✓ pip3: $(pip3 --version)${NC}"
echo

# Step 2: Setup virtual environment
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 2: Setting up Virtual Environment${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

if [ ! -d "venv" ]; then
    echo -e "${YELLOW}Creating virtual environment...${NC}"
    python3 -m venv venv
    echo -e "${GREEN}✓ Virtual environment created${NC}"
else
    echo -e "${GREEN}✓ Virtual environment exists${NC}"
fi

# Activate virtual environment
source venv/bin/activate
echo -e "${GREEN}✓ Virtual environment activated${NC}"
echo

# Step 3: Install Python dependencies
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 3: Installing Python Dependencies${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

if [ -f "examples/requirements.txt" ]; then
    echo -e "${YELLOW}Installing from examples/requirements.txt...${NC}"
    pip3 install -r examples/requirements.txt
    echo -e "${GREEN}✓ Dependencies installed${NC}"
else
    echo -e "${RED}✗ requirements.txt not found${NC}"
    exit 1
fi
echo

# Step 4: Clean up old server processes and sockets
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 4: Cleaning Up Old Processes${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

# Kill any old gunicorn processes
if pgrep -f "gunicorn.*unix_socket_server" >/dev/null; then
    echo -e "${YELLOW}Killing old gunicorn processes...${NC}"
    pkill -f "gunicorn.*unix_socket_server" || true
    sleep 1
fi

# Remove old socket files
if [ -S "$SOCKET_PATH" ]; then
    echo -e "${YELLOW}Removing old socket: $SOCKET_PATH${NC}"
    rm -f "$SOCKET_PATH"
fi

# Remove old PID file
rm -f "$PID_FILE"

echo -e "${GREEN}✓ Cleanup complete${NC}"
echo

# Step 5: Create socket directory
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 5: Preparing Socket Directory${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

mkdir -p "$SOCKET_DIR"
echo -e "${GREEN}✓ Socket directory: $SOCKET_DIR${NC}"
echo

# Step 6: Start Gunicorn server
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 6: Starting Gunicorn Server${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

cd examples
gunicorn \
    --bind "unix:$SOCKET_PATH" \
    --workers 2 \
    --worker-class sync \
    --access-logfile - \
    --error-logfile - \
    --log-level info \
    --timeout 120 \
    --pid "$PID_FILE" \
    unix_socket_server:app &

SERVER_PID=$!
cd ..

echo -e "${GREEN}✓ Gunicorn started (PID: $SERVER_PID)${NC}"
echo

# Step 7: Wait for socket to be ready
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 7: Waiting for Server to be Ready${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

attempts=0
max_attempts=30

while [ ! -S "$SOCKET_PATH" ] && [ $attempts -lt $max_attempts ]; do
    sleep 0.5
    attempts=$((attempts + 1))

    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        echo -e "${RED}✗ Server process died unexpectedly${NC}"
        exit 1
    fi
done

if [ ! -S "$SOCKET_PATH" ]; then
    echo -e "${RED}✗ Timeout waiting for socket${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Server is ready!${NC}"
echo

# Step 8: Test the server
echo -e "${BLUE}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Step 8: Testing Server${NC}"
echo -e "${BLUE}════════════════════════════════════════════════${NC}"

if command -v curl >/dev/null 2>&1; then
    if curl -s --unix-socket "$SOCKET_PATH" http://localhost/ | grep -q "UNIX DOMAIN SOCKET" 2>/dev/null; then
        echo -e "${GREEN}✓ Server responding correctly${NC}"
    else
        echo -e "${YELLOW}⚠ Server response test inconclusive${NC}"
    fi
else
    echo -e "${YELLOW}⚠ curl not available for testing${NC}"
fi
echo

# Display server information
echo -e "${CYAN}╔════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}  ${BOLD}SERVER IS RUNNING${NC}                           ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════╝${NC}"
echo
echo -e "${BOLD}Server Details:${NC}"
echo -e "  Socket Path:    ${GREEN}$SOCKET_PATH${NC}"
echo -e "  PID:            ${GREEN}$SERVER_PID${NC}"
echo -e "  PID File:       ${GREEN}$PID_FILE${NC}"
echo
echo -e "${CYAN}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Servo Client Connection URL:${NC}"
echo -e "${CYAN}════════════════════════════════════════════════${NC}"
echo
echo -e "  ${BOLD}${GREEN}http::unix//localhost/${NC}"
echo
echo -e "Use this URL in Servo's address bar or launch script"
echo
echo -e "${CYAN}════════════════════════════════════════════════${NC}"
echo -e "${BOLD}How to Connect:${NC}"
echo -e "${CYAN}════════════════════════════════════════════════${NC}"
echo
echo -e "${YELLOW}Option 1:${NC} Run Servo setup script in another terminal:"
echo -e "  ${BLUE}./setup_and_run_servo.sh${NC}"
echo
echo -e "${YELLOW}Option 2:${NC} Run Servo manually:"
echo -e "  ${BLUE}./target/debug/servo http::unix//localhost/${NC}"
echo
echo -e "${YELLOW}Option 3:${NC} Test with curl:"
echo -e "  ${BLUE}curl --unix-socket $SOCKET_PATH http://localhost/${NC}"
echo
echo -e "${CYAN}════════════════════════════════════════════════${NC}"
echo
echo -e "${BOLD}${YELLOW}Server is running. Logs will appear below:${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"
echo
echo -e "${CYAN}════════════════════════════════════════════════${NC}"
echo

# Keep running and show logs (server logs will appear in this terminal)
wait $SERVER_PID
