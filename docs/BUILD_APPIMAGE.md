# Building AppImage for Sqrilizz Launcher

## Prerequisites

### Ubuntu/Debian

```bash
sudo apt update
sudo apt install -y \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.1-dev \
    patchelf
```

### Fedora

```bash
sudo dnf install -y \
    gcc \
    gcc-c++ \
    make \
    openssl-devel \
    gtk3-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    webkit2gtk4.1-devel \
    patchelf
```

### Arch Linux

```bash
sudo pacman -S --needed \
    base-devel \
    curl \
    wget \
    file \
    openssl \
    gtk3 \
    libappindicator-gtk3 \
    librsvg \
    webkit2gtk-4.1 \
    patchelf
```

## Install Rust and Node.js

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Node.js (via nvm)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install 20
nvm use 20

# Install pnpm
npm install -g pnpm
```

## Build Steps

### 1. Clone and Install Dependencies

```bash
git clone https://github.com/yourusername/sqrilizz-launcher.git
cd sqrilizz-launcher
pnpm install
```

### 2. Build AppImage

```bash
cd apps/app
pnpm tauri build -- --bundles appimage
```

### 3. Output Location

The AppImage will be in:

```
apps/app/target/release/bundle/appimage/sqrilizz-launcher_1.0.0_amd64.AppImage
```

## Testing AppImage

```bash
# Make executable
chmod +x sqrilizz-launcher_1.0.0_amd64.AppImage

# Run
./sqrilizz-launcher_1.0.0_amd64.AppImage
```

## Troubleshooting

### "error: failed to bundle project"

Make sure all dependencies are installed:

```bash
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev
```

### "GLIBC version not found"

Build on an older Ubuntu version (20.04) for better compatibility:

```bash
docker run -it --rm -v $(pwd):/workspace ubuntu:20.04
# Then install dependencies and build inside container
```

### AppImage doesn't run on other systems

This usually means missing libraries. Use `bundleMediaFramework: true` in config (already set).

### "Failed to load module 'canberra-gtk-module'"

This is a warning and can be ignored. To fix:

```bash
sudo apt install libcanberra-gtk-module
```

## Distribution

### Upload to GitHub Releases

```bash
gh release create v1.0.0 \
  apps/app/target/release/bundle/appimage/sqrilizz-launcher_1.0.0_amd64.AppImage \
  --title "Sqrilizz Launcher v1.0.0" \
  --notes "First release"
```

### Make it executable by default

Users need to run:

```bash
chmod +x sqrilizz-launcher_1.0.0_amd64.AppImage
```

Or integrate with system:

```bash
# Extract and install
./sqrilizz-launcher_1.0.0_amd64.AppImage --appimage-extract
sudo mv squashfs-root /opt/sqrilizz-launcher
sudo ln -s /opt/sqrilizz-launcher/AppRun /usr/local/bin/sqrilizz-launcher
```

## Build for Multiple Architectures

### x86_64 (default)

```bash
pnpm tauri build -- --bundles appimage
```

### ARM64 (requires ARM64 system or cross-compilation)

```bash
rustup target add aarch64-unknown-linux-gnu
pnpm tauri build -- --target aarch64-unknown-linux-gnu --bundles appimage
```

## Notes

- AppImage includes all dependencies (self-contained)
- Works on most Linux distributions
- No installation required
- Portable - can run from USB drive
- First build takes 10-30 minutes
- Subsequent builds are faster (5-10 minutes)

## Compatibility

The AppImage should work on:

- Ubuntu 20.04+
- Debian 11+
- Fedora 35+
- Arch Linux (latest)
- openSUSE Leap 15.4+
- Most other modern Linux distributions

For maximum compatibility, build on Ubuntu 20.04.
