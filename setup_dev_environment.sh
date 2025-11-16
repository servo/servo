#!/bin/bash
# Development Environment Setup for Servo UDS on Ubuntu 25.10
# This script installs all dependencies needed to build and test Servo with Unix socket support

set -e  # Exit on error

echo "=========================================="
echo "Servo UDS Development Environment Setup"
echo "Ubuntu 25.10"
echo "=========================================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if running on Ubuntu
if ! grep -q "Ubuntu" /etc/os-release; then
    echo -e "${YELLOW}⚠ Warning: This script is designed for Ubuntu 25.10${NC}"
    echo "You may need to adjust package names for your distribution"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check Ubuntu version
UBUNTU_VERSION=$(lsb_release -rs)
echo "Detected Ubuntu version: $UBUNTU_VERSION"
echo

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install packages
install_packages() {
    echo -e "${BLUE}Installing system packages...${NC}"
    sudo apt-get update

    # Servo build dependencies
    sudo apt-get install -y \
        git \
        curl \
        build-essential \
        cmake \
        pkg-config \
        libssl-dev \
        libfreetype6-dev \
        libfontconfig1-dev \
        libxcb-render0-dev \
        libxcb-shape0-dev \
        libxcb-xfixes0-dev \
        libxcb-composite0-dev \
        libharfbuzz-dev \
        libglib2.0-dev \
        libgstreamer1.0-dev \
        libgstreamer-plugins-base1.0-dev \
        libgstreamer-plugins-bad1.0-dev \
        gstreamer1.0-plugins-good \
        gstreamer1.0-plugins-bad \
        gstreamer1.0-plugins-ugly \
        gstreamer1.0-libav \
        libegl1-mesa-dev \
        libgl1-mesa-dev \
        libdbus-1-dev \
        libudev-dev \
        llvm-dev \
        libclang-dev \
        clang \
        autoconf2.13 \
        ccache \
        python3 \
        python3-pip \
        python3-venv

    echo -e "${GREEN}✓ System packages installed${NC}"

    # Ubuntu 25.10+ workaround: Create missing libudev.pc file
    if [ ! -f /usr/lib/x86_64-linux-gnu/pkgconfig/libudev.pc ]; then
        echo -e "${BLUE}Creating libudev.pc workaround for Ubuntu 25.10+...${NC}"
        sudo tee /usr/lib/x86_64-linux-gnu/pkgconfig/libudev.pc > /dev/null <<'EOF'
prefix=/usr
exec_prefix=${prefix}
libdir=${exec_prefix}/lib/x86_64-linux-gnu
includedir=${prefix}/include

Name: libudev
Description: libudev - systemd udev library
Version: 257.9
Libs: -L${libdir} -ludev
Cflags: -I${includedir}
EOF
        echo -e "${GREEN}✓ libudev.pc created${NC}"
    fi
    echo
}

# Function to install Rust
install_rust() {
    if command_exists rustc; then
        echo -e "${GREEN}✓ Rust already installed: $(rustc --version)${NC}"
    else
        echo -e "${BLUE}Installing Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        echo -e "${GREEN}✓ Rust installed${NC}"
    fi

    # Ensure we're on the right Rust version for Servo
    echo -e "${BLUE}Setting Rust toolchain...${NC}"
    rustup default stable
    rustup component add rustfmt clippy
    echo -e "${GREEN}✓ Rust toolchain configured${NC}"
    echo
}

# Function to install Python dependencies
install_python_deps() {
    echo -e "${BLUE}Installing Python dependencies...${NC}"

    # Create virtual environment
    if [ ! -d "venv" ]; then
        python3 -m venv venv
    fi

    # Activate and install
    source venv/bin/activate

    pip install --upgrade pip
    pip install \
        flask \
        gunicorn \
        requests \
        requests-unixsocket

    deactivate

    echo -e "${GREEN}✓ Python dependencies installed in ./venv${NC}"
    echo
}

# Function to check/install network tools
install_network_tools() {
    echo -e "${BLUE}Installing network monitoring tools...${NC}"

    sudo apt-get install -y \
        net-tools \
        iproute2 \
        lsof \
        netcat-openbsd \
        curl

    echo -e "${GREEN}✓ Network tools installed${NC}"
    echo
}

# Function to setup git hooks (optional)
setup_git_hooks() {
    echo -e "${BLUE}Setting up git hooks...${NC}"

    # Pre-commit hook for running tests
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Run quick checks before commit
echo "Running pre-commit checks..."

# Check Rust formatting
cargo fmt --all -- --check
if [ $? -ne 0 ]; then
    echo "❌ Code formatting check failed. Run: cargo fmt --all"
    exit 1
fi

# Run unit tests for transport_url
cargo test --lib -p net transport_url::tests --quiet
if [ $? -ne 0 ]; then
    echo "❌ Unit tests failed"
    exit 1
fi

echo "✓ Pre-commit checks passed"
EOF

    chmod +x .git/hooks/pre-commit
    echo -e "${GREEN}✓ Git hooks configured${NC}"
    echo
}

# Function to verify installation
verify_installation() {
    echo -e "${BLUE}Verifying installation...${NC}"
    echo

    local all_good=true

    # Check Rust
    if command_exists rustc; then
        echo -e "${GREEN}✓ Rust: $(rustc --version)${NC}"
    else
        echo -e "${RED}✗ Rust not found${NC}"
        all_good=false
    fi

    # Check Cargo
    if command_exists cargo; then
        echo -e "${GREEN}✓ Cargo: $(cargo --version)${NC}"
    else
        echo -e "${RED}✗ Cargo not found${NC}"
        all_good=false
    fi

    # Check Python
    if command_exists python3; then
        echo -e "${GREEN}✓ Python: $(python3 --version)${NC}"
    else
        echo -e "${RED}✗ Python3 not found${NC}"
        all_good=false
    fi

    # Check Flask (in venv)
    if [ -f "venv/bin/activate" ]; then
        source venv/bin/activate
        if python -c "import flask" 2>/dev/null; then
            echo -e "${GREEN}✓ Flask: $(python -c 'import flask; print(flask.__version__)')${NC}"
        else
            echo -e "${RED}✗ Flask not found in venv${NC}"
            all_good=false
        fi

        if python -c "import gunicorn" 2>/dev/null; then
            echo -e "${GREEN}✓ Gunicorn installed${NC}"
        else
            echo -e "${RED}✗ Gunicorn not found in venv${NC}"
            all_good=false
        fi
        deactivate
    else
        echo -e "${RED}✗ Virtual environment not found${NC}"
        all_good=false
    fi

    # Check network tools
    if command_exists ss; then
        echo -e "${GREEN}✓ ss (iproute2) installed${NC}"
    else
        echo -e "${RED}✗ ss not found${NC}"
        all_good=false
    fi

    if command_exists lsof; then
        echo -e "${GREEN}✓ lsof installed${NC}"
    else
        echo -e "${RED}✗ lsof not found${NC}"
        all_good=false
    fi

    echo

    if [ "$all_good" = true ]; then
        echo -e "${GREEN}=========================================="
        echo "✓ All dependencies installed successfully!"
        echo "==========================================${NC}"
        return 0
    else
        echo -e "${RED}=========================================="
        echo "✗ Some dependencies are missing"
        echo "==========================================${NC}"
        return 1
    fi
}

# Function to print next steps
print_next_steps() {
    echo
    echo -e "${BLUE}=========================================="
    echo "Next Steps"
    echo "==========================================${NC}"
    echo
    echo "1. Activate Python virtual environment:"
    echo "   source venv/bin/activate"
    echo
    echo "2. Build Servo:"
    echo "   ./mach build -d"
    echo
    echo "3. Run tests:"
    echo "   ./run_all_tests.sh"
    echo
    echo "4. Launch demo:"
    echo "   ./launch_demo.sh"
    echo
    echo "5. Verify no TCP connections:"
    echo "   ./verify_no_tcp.sh"
    echo
    echo -e "${YELLOW}Note: Building Servo for the first time may take 30-60 minutes.${NC}"
    echo
}

# Main installation flow
main() {
    echo "This script will install:"
    echo "  - Servo build dependencies (compilers, libraries)"
    echo "  - Rust toolchain"
    echo "  - Python virtual environment with Flask/Gunicorn"
    echo "  - Network monitoring tools (ss, lsof, etc.)"
    echo "  - Git hooks for pre-commit checks"
    echo
    read -p "Continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled"
        exit 0
    fi

    install_packages
    install_rust
    install_python_deps
    install_network_tools

    if [ -d ".git" ]; then
        read -p "Setup git hooks? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            setup_git_hooks
        fi
    fi

    verify_installation
    local verify_status=$?

    print_next_steps

    exit $verify_status
}

# Run main
main
