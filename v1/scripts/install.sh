#!/bin/bash
# scripts/install.sh
# Installation script for Unix-like systems (Linux/macOS)

set -e

echo "========================================"
echo "Git Multi-Account Manager Installation"
echo "========================================"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check Python version
echo "Checking Python version..."
PYTHON_VERSION=$(python3 --version 2>&1 | grep -oP '\d+\.\d+')
REQUIRED_VERSION="3.9"

if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$PYTHON_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
    echo -e "${RED}Error: Python 3.9 or higher is required${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Python $PYTHON_VERSION found${NC}"

# Check Git
echo "Checking Git..."
if ! command -v git &> /dev/null; then
    echo -e "${RED}Error: Git is not installed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Git found${NC}"

# Create virtual environment
echo ""
echo "Creating virtual environment..."
python3 -m venv git_venv
source git_venv/bin/activate
echo -e "${GREEN}✓ Virtual environment created${NC}"

# Upgrade pip
echo ""
echo "Upgrading pip..."
pip install --upgrade pip
echo -e "${GREEN}✓ pip upgraded${NC}"

# Install package
echo ""
echo "Installing Git Manager..."
pip install -e .
echo -e "${GREEN}✓ Git Manager installed${NC}"

# Create configuration directory
echo ""
echo "Setting up configuration..."
CONFIG_DIR="$HOME/.git-manager"
mkdir -p "$CONFIG_DIR"
echo -e "${GREEN}✓ Configuration directory created${NC}"

# Copy example configurations
if [ -f "config/config.example.json" ]; then
    cp config/config.example.json "$CONFIG_DIR/config.json"
    echo -e "${GREEN}✓ Config file created${NC}"
fi

if [ -f "config/accounts.example.json" ]; then
    cp config/accounts.example.json "$CONFIG_DIR/accounts.example.json"
    echo -e "${YELLOW}! Example accounts file created at $CONFIG_DIR/accounts.example.json${NC}"
fi

# Create desktop entry (Linux only)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    DESKTOP_FILE="$HOME/.local/share/applications/git-manager.desktop"
    mkdir -p "$(dirname "$DESKTOP_FILE")"
    
    cat > "$DESKTOP_FILE" << EOF
[Desktop Entry]
Name=Git Manager
Comment=Manage multiple Git accounts
Exec=$(which git-manager-desktop)
Icon=git
Terminal=false
Type=Application
Categories=Development;
EOF
    
    echo -e "${GREEN}✓ Desktop entry created${NC}"
fi

# Setup logging
echo ""
echo "Setting up logging..."
if [ "$EUID" -eq 0 ]; then
    touch /var/log/git-manager.log
    chmod 666 /var/log/git-manager.log
    echo -e "${GREEN}✓ Log file created${NC}"
else
    echo -e "${YELLOW}! Run 'sudo $(basename $0)' to enable system-wide logging${NC}"
fi

echo ""
echo -e "${GREEN}========================================"
echo "Installation complete!"
echo "========================================${NC}"
echo ""
echo "To get started:"
echo "  1. Activate virtual environment: source venv/bin/activate"
echo "  2. Run CLI: git-manager"
echo "  3. Run Web: git-manager-web"
echo "  4. Run Desktop: git-manager-desktop"
echo ""
echo "Configuration files are in: $CONFIG_DIR"
echo ""