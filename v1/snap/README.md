# Snap Build Configuration

This directory contains the configuration for building Git Multi-Account Manager as a Snap package.

## Building the Snap

### Prerequisites

- Install snapcraft:
  ```bash
  sudo apt install snapcraft
  ```

- Install multipass (for building in a container):
  ```bash
  sudo apt install multipass
  ```

### Build Commands

**Build the snap:**
```bash
snapcraft
```

**Build without multipass (requires all build dependencies):**
```bash
snapcraft --use-lxd
```

**Build for specific architecture:**
```bash
snapcraft --build-for=amd64
snapcraft --build-for=arm64
```

**Clean build:**
```bash
snapcraft clean
snapcraft
```

## Installation

### From local build:
```bash
sudo snap install --dangerous ./git-manager_*.snap
```

### From Snap Store (once published):
```bash
sudo snap install git-manager
```

## Usage

### CLI Interface:
```bash
git-manager --help
git-manager get
git-manager set <account>
git-manager list
```

### Daemon Mode:
```bash
git-manager-daemon
```

## Permissions

The snap requires the following permissions:
- **home**: Access to home directory for configuration and SSH keys
- **network**: Network access for Git operations
- **network-bind**: Bind to network interfaces
- **ssh-keys**: Access to SSH keys in ~/.ssh
- **dot-ssh-config**: Access to SSH configuration files

## Configuration

Configuration files are stored in `~/.git-manager/`:
- `config.json` - Main configuration
- `accounts.json` - Account definitions

Example configuration files are provided in `/etc/git-manager/` within the snap.

## Troubleshooting

### Permission Denied Errors
If you encounter permission errors with SSH keys:
```bash
sudo snap connect git-manager:ssh-keys
sudo snap connect git-manager:dot-ssh-config
```

### Check Snap Logs
```bash
sudo snap logs git-manager -f
```

### Verify Installation
```bash
snap info git-manager
snap list git-manager
```

## Building for Distribution

### Prepare for Snap Store:

1. Update version in `snapcraft.yaml`
2. Build the snap:
   ```bash
   snapcraft
   ```
3. Test the snap:
   ```bash
   sudo snap install --dangerous ./git-manager_*.snap
   ```
4. Submit to Snap Store:
   ```bash
   snapcraft upload ./git-manager_*.snap
   ```

## File Structure

```
snap/
├── snapcraft.yaml      # Main snap configuration
├── local/
│   └── git-manager.desktop  # Desktop entry file
└── README.md           # This file
```

## Additional Resources

- [Snapcraft Documentation](https://snapcraft.io/docs)
- [Snap Store](https://snapcraft.io/store)
- [Snap Confinement](https://snapcraft.io/docs/snap-confinement)
