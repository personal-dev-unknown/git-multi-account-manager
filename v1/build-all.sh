#!/bin/bash
set -e

# Create output directories
mkdir -p dist/linux dist/windows

echo "=== Building Linux version ==="
docker build -f Dockerfile.linux -t git-manager-linux .

# For Windows build, we need to ensure the Windows container is ready
echo -e "\n=== Building Windows version ==="
# Convert paths to Windows-style for the container
WIN_PATH=$(pwd | sed 's/^\/mnt\/\([a-z]\+\)\//\U\1:\\/' | tr '/' '\\')
echo "Building Windows executable in $WIN_PATH"

docker build -f Dockerfile.windows -t git-manager-windows .

echo -e "\n=== Build Complete ==="
ls -l dist/*
echo -e "\nExecutables have been built in the 'dist' directory:"
echo "- Linux:   dist/linux/git-manager"
echo "- Windows: dist/windows/git-manager.exe"
