#!/bin/bash
# scripts/build-snap.sh
# Automated snap building and testing script for Git Manager

set -e

# ============================================
# Configuration
# ============================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${PROJECT_ROOT}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Read version
if [ -f VERSION ]; then
    VERSION=$(cat VERSION | tr -d ' \n')
else
    VERSION="2.1.0"
    echo -e "${YELLOW}⚠️  VERSION file not found, using default: ${VERSION}${NC}"
fi

# ============================================
# Helper Functions
# ============================================

print_header() {
    echo ""
    echo -e "${CYAN}======================================${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}======================================${NC}"
    echo ""
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

check_command() {
    if ! command -v "$1" &> /dev/null; then
        print_error "$1 is not installed"
        return 1
    fi
    return 0
}

# ============================================
# Pre-flight Checks
# ============================================

preflight_checks() {
    print_header "Pre-flight Checks"
    
    local errors=0
    
    # Check for required commands
    print_info "Checking required tools..."
    
    if check_command snapcraft; then
        SNAPCRAFT_VERSION=$(snapcraft --version 2>&1 | head -1)
        print_success "snapcraft: ${SNAPCRAFT_VERSION}"
    else
        print_error "snapcraft not found"
        echo "  Install with: sudo snap install snapcraft --classic"
        ((errors++))
    fi
    
    if check_command lxd; then
        print_success "lxd: available"
    else
        print_warning "lxd not found (optional, but recommended for clean builds)"
        echo "  Install with: sudo snap install lxd && sudo lxd init --auto"
    fi
    
    if check_command python3; then
        PYTHON_VERSION=$(python3 --version)
        print_success "python3: ${PYTHON_VERSION}"
    else
        print_error "python3 not found"
        ((errors++))
    fi
    
    # Check for required files
    print_info "Checking project structure..."
    
    required_files=(
        "snap/snapcraft.yaml"
        "setup.py"
        "src/git_manager/__init__.py"
        "snap/README.md"
        "snap/hooks/install"
        "snap/hooks/configure"
        "snap/hooks/post-refresh"
    )
    
    for file in "${required_files[@]}"; do
        if [ -f "${file}" ] || [ -d "${file}" ]; then
            print_success "${file}"
        else
            print_error "${file} not found"
            ((errors++))
        fi
    done
    
    # Check snap directory structure
    if [ -d "snap/local/scripts" ]; then
        print_success "snap/local/scripts/"
    else
        print_warning "snap/local/scripts/ not found - creating"
        mkdir -p snap/local/scripts
    fi
    
    # Check for hooks
    if [ -d "snap/hooks" ]; then
        print_success "snap/hooks/"
        for hook in install configure post-refresh; do
            if [ -f "snap/hooks/${hook}" ]; then
                print_success "  • ${hook}"
            else
                print_warning "  • ${hook} not found"
            fi
        done
    else
        print_warning "snap/hooks/ not found"
    fi
    
    if [ $errors -gt 0 ]; then
        print_error "${errors} error(s) found. Please fix them before building."
        return 1
    fi
    
    print_success "All pre-flight checks passed!"
    return 0
}

# ============================================
# Setup Snap Directory Structure
# ============================================

setup_snap_structure() {
    print_header "Setting up Snap Structure"
    
    # Create directories
    mkdir -p snap/local/scripts
    mkdir -p snap/hooks
    mkdir -p snap/local
    
    # Ensure hooks are executable
    if [ -d snap/hooks ]; then
        chmod +x snap/hooks/* 2>/dev/null || true
    fi
    
    # Ensure setup-env is executable
    if [ -f snap/local/scripts/setup-env ]; then
        chmod +x snap/local/scripts/setup-env
    fi
    
    print_success "Snap structure ready"
}

# ============================================
# Clean Build Artifacts
# ============================================

clean_build() {
    print_header "Cleaning Build Artifacts"
    
    if [ -d "parts" ]; then
        print_info "Removing parts/"
        rm -rf parts
    fi
    
    if [ -d "stage" ]; then
        print_info "Removing stage/"
        rm -rf stage
    fi
    
    if [ -d "prime" ]; then
        print_info "Removing prime/"
        rm -rf prime
    fi
    
    # Remove old snap files
    if ls *.snap 1> /dev/null 2>&1; then
        print_info "Removing old .snap files"
        rm -f *.snap
    fi
    
    # Clean Python cache
    find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
    find . -type f -name "*.pyc" -delete 2>/dev/null || true
    
    print_success "Clean complete"
}

# ============================================
# Build Snap
# ============================================

build_snap() {
    local use_lxd=$1
    local target_arch=$2
    
    print_header "Building Snap Package"
    
    print_info "Version: ${VERSION}"
    print_info "Architecture: ${target_arch:-native}"
    print_info "Clean build: ${use_lxd}"
    
    local build_cmd="snapcraft pack"
    
    if [ "${use_lxd}" = "true" ]; then
        build_cmd="${build_cmd} --use-lxd"
    fi
    
    if [ -n "${target_arch}" ]; then
        build_cmd="${build_cmd} --target-arch=${target_arch}"
    fi
    
    print_info "Running: ${build_cmd}"
    echo ""
    
    if ${build_cmd}; then
        print_success "Snap built successfully!"
        
        # Find the created snap file
        SNAP_FILE=$(ls -t *.snap 2>/dev/null | head -1)
        if [ -n "${SNAP_FILE}" ]; then
            SNAP_SIZE=$(du -h "${SNAP_FILE}" | cut -f1)
            print_info "Snap file: ${SNAP_FILE}"
            print_info "Size: ${SNAP_SIZE}"
        fi
        
        return 0
    else
        print_error "Build failed"
        return 1
    fi
}

# ============================================
# Install and Test Snap
# ============================================

test_snap() {
    print_header "Testing Snap Installation"
    
    SNAP_FILE=$(ls -t *.snap 2>/dev/null | head -1)
    
    if [ -z "${SNAP_FILE}" ]; then
        print_error "No snap file found"
        return 1
    fi
    
    print_info "Installing: ${SNAP_FILE}"
    
    # Remove existing installation
    if snap list git-manager &>/dev/null; then
        print_info "Removing existing installation..."
        sudo snap remove git-manager
    fi
    
    # Install the snap
    if sudo snap install --dangerous "${SNAP_FILE}"; then
        print_success "Snap installed successfully"
    else
        print_error "Installation failed"
        return 1
    fi
    
    # Connect interfaces
    print_info "Connecting interfaces..."
    sudo snap connect git-manager:ssh-keys 2>/dev/null || print_warning "Could not connect ssh-keys"
    sudo snap connect git-manager:dot-ssh-config 2>/dev/null || print_warning "Could not connect dot-ssh-config"
    sudo snap connect git-manager:dot-gitconfig 2>/dev/null || print_warning "Could not connect dot-gitconfig"
    
    # Run basic tests
    print_header "Running Basic Tests"
    
    echo -e "${CYAN}Test 1: Version check${NC}"
    if git-manager.version; then
        print_success "Version check passed"
    else
        print_error "Version check failed"
        return 1
    fi
    
    echo ""
    echo -e "${CYAN}Test 2: Help command${NC}"
    if git-manager --help &>/dev/null; then
        print_success "Help command passed"
    else
        print_error "Help command failed"
        return 1
    fi
    
    echo ""
    echo -e "${CYAN}Test 3: Doctor/diagnostics${NC}"
    if git-manager.doctor &>/dev/null; then
        print_success "Doctor check passed"
    else
        print_warning "Doctor check had warnings (may be expected)"
    fi
    
    echo ""
    echo -e "${CYAN}Test 4: Configuration directory${NC}"
    CONFIG_DIR="${HOME}/snap/git-manager/current/.config/git-manager"
    if [ -d "${CONFIG_DIR}" ]; then
        print_success "Configuration directory created"
        ls -la "${CONFIG_DIR}" || true
    else
        print_warning "Configuration directory not found"
    fi
    
    print_success "All tests passed!"
    
    return 0
}

# ============================================
# Generate Build Report
# ============================================

generate_report() {
    print_header "Build Report"
    
    SNAP_FILE=$(ls -t *.snap 2>/dev/null | head -1)
    
    if [ -z "${SNAP_FILE}" ]; then
        print_error "No snap file found"
        return 1
    fi
    
    echo "📦 Snap Package Information"
    echo "  File: ${SNAP_FILE}"
    echo "  Size: $(du -h "${SNAP_FILE}" | cut -f1)"
    echo ""
    
    echo "📊 Snap Details"
    snap info "${SNAP_FILE}" --verbose 2>/dev/null || true
    echo ""
    
    if snap list git-manager &>/dev/null; then
        echo "✅ Installation Status"
        snap list git-manager
        echo ""
        
        echo "🔌 Connected Interfaces"
        snap connections git-manager
        echo ""
        
        echo "💾 Disk Usage"
        snap info git-manager --verbose | grep -E "(installed|size)" || true
        echo ""
    fi
    
    echo "📍 File Location"
    echo "  ${SNAP_FILE}"
    echo ""
    
    echo "🚀 Next Steps"
    echo "  • Test: git-manager --help"
    echo "  • Upload: snapcraft upload ${SNAP_FILE} --release=edge"
    echo "  • Install elsewhere: sudo snap install --dangerous ${SNAP_FILE}"
    echo ""
}

# ============================================
# Upload to Snap Store
# ============================================

upload_snap() {
    local channel=$1
    
    print_header "Uploading to Snap Store"
    
    SNAP_FILE=$(ls -t *.snap 2>/dev/null | head -1)
    
    if [ -z "${SNAP_FILE}" ]; then
        print_error "No snap file found"
        return 1
    fi
    
    if [ -z "${channel}" ]; then
        print_error "Channel not specified"
        echo "Usage: upload [edge|beta|candidate|stable]"
        return 1
    fi
    
    print_info "Uploading ${SNAP_FILE} to ${channel} channel..."
    
    if snapcraft upload "${SNAP_FILE}" --release="${channel}"; then
        print_success "Upload successful!"
        echo ""
        print_info "Monitor release: snapcraft status git-manager"
    else
        print_error "Upload failed"
        return 1
    fi
}

# ============================================
# Main Menu
# ============================================

show_menu() {
    echo ""
    echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║   Git Manager Snap Build Script       ║${NC}"
    echo -e "${CYAN}║   Version: ${VERSION}                       ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════╝${NC}"
    echo ""
    echo "1) Pre-flight checks"
    echo "2) Clean build artifacts"
    echo "3) Build snap (local)"
    echo "4) Build snap (clean with LXD)"
    echo "5) Build for specific architecture"
    echo "6) Install and test"
    echo "7) Generate build report"
    echo "8) Upload to Snap Store"
    echo "9) Full build and test pipeline"
    echo "0) Exit"
    echo ""
}

# ============================================
# Full Pipeline
# ============================================

full_pipeline() {
    print_header "Full Build and Test Pipeline"
    
    preflight_checks || return 1
    setup_snap_structure || return 1
    clean_build || return 1
    build_snap "true" "" || return 1
    test_snap || return 1
    generate_report || return 1
    
    print_success "Pipeline completed successfully!"
}

# ============================================
# Command Line Arguments
# ============================================

if [ $# -gt 0 ]; then
    case "$1" in
        check|preflight)
            preflight_checks
            ;;
        clean)
            clean_build
            ;;
        build)
            setup_snap_structure
            build_snap "${2:-false}" "$3"
            ;;
        test)
            test_snap
            ;;
        report)
            generate_report
            ;;
        upload)
            upload_snap "$2"
            ;;
        pipeline|full)
            full_pipeline
            ;;
        *)
            echo "Usage: $0 [check|clean|build|test|report|upload|pipeline]"
            exit 1
            ;;
    esac
    exit $?
fi

# ============================================
# Interactive Menu
# ============================================

while true; do
    show_menu
    read -p "Select option: " choice
    echo ""
    
    case $choice in
        1)
            preflight_checks
            ;;
        2)
            clean_build
            ;;
        3)
            setup_snap_structure
            build_snap "false" ""
            ;;
        4)
            setup_snap_structure
            build_snap "true" ""
            ;;
        5)
            read -p "Enter architecture (amd64/arm64/armhf): " arch
            setup_snap_structure
            build_snap "true" "$arch"
            ;;
        6)
            test_snap
            ;;
        7)
            generate_report
            ;;
        8)
            read -p "Enter channel (edge/beta/candidate/stable): " channel
            upload_snap "$channel"
            ;;
        9)
            full_pipeline
            ;;
        0)
            echo "Goodbye!"
            exit 0
            ;;
        *)
            print_error "Invalid option"
            ;;
    esac
    
    echo ""
    read -p "Press Enter to continue..."
done