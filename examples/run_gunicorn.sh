#!/bin/bash
#
# Run Gunicorn server on Unix domain socket
#
# This script starts a Gunicorn server that listens on a Unix socket
# instead of a TCP port, for use with Servo's Unix socket networking mode.
#

set -e

# Configuration
SOCKET_DIR="${SERVO_SOCKET_DIR:-/tmp/servo-sockets}"
SOCKET_PATH="$SOCKET_DIR/localhost.sock"
APP_MODULE="unix_socket_server:app"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Gunicorn Unix Socket Server${NC}"
echo -e "${BLUE}================================${NC}"
echo

# Check if Python 3 is installed
if ! command -v python3 &> /dev/null; then
    echo -e "${YELLOW}Error: python3 is not installed${NC}"
    exit 1
fi

# Check if gunicorn is installed
if ! python3 -c "import gunicorn" 2>/dev/null; then
    echo -e "${YELLOW}Gunicorn not found. Installing dependencies...${NC}"
    echo
    pip3 install flask gunicorn
    echo
fi

# Create socket directory if it doesn't exist
echo -e "${GREEN}Creating socket directory: $SOCKET_DIR${NC}"
mkdir -p "$SOCKET_DIR"

# Clean up old socket if it exists
if [ -S "$SOCKET_PATH" ]; then
    echo -e "${YELLOW}Removing existing socket: $SOCKET_PATH${NC}"
    rm -f "$SOCKET_PATH"
fi

echo -e "${GREEN}Starting Gunicorn server...${NC}"
echo -e "  Socket: ${BLUE}$SOCKET_PATH${NC}"
echo -e "  App:    ${BLUE}$APP_MODULE${NC}"
echo

# Change to the examples directory
cd "$SCRIPT_DIR"

# Run Gunicorn with Unix socket
echo -e "${GREEN}Server is running!${NC}"
echo
echo -e "To test with Servo, run in another terminal:"
echo -e "  ${BLUE}./servo --unix-sockets --socket-dir=\"$SOCKET_DIR\" --socket-mapping=\"localhost:$SOCKET_PATH\" -z http://localhost/${NC}"
echo
echo -e "Press Ctrl+C to stop the server"
echo
echo "========================================"

# Set socket permissions for broader access (optional)
# This allows any user to connect to the socket
exec gunicorn \
    --bind "unix:$SOCKET_PATH" \
    --workers 2 \
    --worker-class sync \
    --access-logfile - \
    --error-logfile - \
    --log-level info \
    --timeout 120 \
    "$APP_MODULE"
