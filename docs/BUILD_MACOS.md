# Building Sqrilizz Launcher for macOS

## Prerequisites

1. **macOS** (required for building macOS apps)
2. **Xcode Command Line Tools**:
   ```bash
   xcode-select --install
   ```
3. **Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
4. **Node.js** (v20+) and **pnpm**:
   ```bash
   brew install node
   npm install -g pnpm
   ```

## Build Steps

### 1. Install Dependencies

```bash
pnpm install
```

### 2. Build for macOS

```bash
# Development build (faster, larger)
pnpm app:build

# Release build (optimized, smaller)
cd apps/app
pnpm tauri build
```

### 3. Output Location

The built app will be in:

```
apps/app/target/release/bundle/macos/Sqrilizz Launcher.app
```

Or DMG installer:

```
apps/app/target/release/bundle/dmg/Sqrilizz Launcher_1.1.0_aarch64.dmg
apps/app/target/release/bundle/dmg/Sqrilizz Launcher_1.1.0_x64.dmg
```

## Architecture Support

### Universal Binary (Intel + Apple Silicon)

To build a universal binary that works on both Intel and Apple Silicon Macs:

```bash
# Add targets
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin

# Build universal
cd apps/app
pnpm tauri build -- --target universal-apple-darwin
```

### Apple Silicon Only (M1/M2/M3)

```bash
rustup target add aarch64-apple-darwin
cd apps/app
pnpm tauri build -- --target aarch64-apple-darwin
```

### Intel Only

```bash
rustup target add x86_64-apple-darwin
cd apps/app
pnpm tauri build -- --target x86_64-apple-darwin
```

## Code Signing (Optional but Recommended)

For distribution outside the App Store, you need to sign the app:

### 1. Get Apple Developer Certificate

- Sign up for [Apple Developer Program](https://developer.apple.com) ($99/year)
- Create a Developer ID Application certificate in Xcode

### 2. Configure Signing

Edit `apps/app/tauri.conf.json`:

```json
{
	"bundle": {
		"macOS": {
			"signingIdentity": "Developer ID Application: Your Name (TEAM_ID)"
		}
	}
}
```

### 3. Build with Signing

```bash
cd apps/app
pnpm tauri build
```

## Notarization (For Distribution)

To distribute outside the App Store, you need to notarize:

### 1. Create App-Specific Password

- Go to [appleid.apple.com](https://appleid.apple.com)
- Generate an app-specific password

### 2. Notarize

```bash
# Store credentials
xcrun notarytool store-credentials "notarytool-profile" \
  --apple-id "your@email.com" \
  --team-id "TEAM_ID" \
  --password "app-specific-password"

# Notarize the DMG
xcrun notarytool submit "Sqrilizz Launcher_1.1.0_aarch64.dmg" \
  --keychain-profile "notarytool-profile" \
  --wait

# Staple the notarization
xcrun stapler staple "Sqrilizz Launcher_1.1.0_aarch64.dmg"
```

## Troubleshooting

### "xcrun: error: invalid active developer path"

```bash
xcode-select --install
```

### "error: linker `cc` not found"

```bash
xcode-select --install
```

### Build fails with "framework not found"

Make sure Xcode Command Line Tools are installed:

```bash
xcode-select -p
# Should output: /Library/Developer/CommandLineTools
```

### App crashes on launch

Check Console.app for crash logs:

```bash
open /Applications/Utilities/Console.app
```

## Quick Build (No Signing)

For testing only (won't run on other Macs):

```bash
pnpm app:build
open apps/app/target/release/bundle/macos/Sqrilizz\ Launcher.app
```

## Distribution

### DMG (Recommended)

The build process automatically creates a DMG installer.

### ZIP

```bash
cd apps/app/target/release/bundle/macos
zip -r "Sqrilizz Launcher.zip" "Sqrilizz Launcher.app"
```

### App Bundle

Just distribute the `.app` folder directly.

## Notes

- **First build takes 10-30 minutes** (compiling Rust dependencies)
- **Subsequent builds are faster** (5-10 minutes)
- **Universal builds take longer** (building for 2 architectures)
- **Signing requires Apple Developer account** ($99/year)
- **Notarization is required** for distribution outside App Store
