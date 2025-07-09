#!/bin/bash

# Cross-platform release build script for Rusty
# This script builds release binaries for all supported platforms

set -e

echo "🚀 Building Rusty for all platforms..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create release directory
mkdir -p releases

# Function to build for a specific target
build_target() {
    local target=$1
    local name=$2
    
    echo -e "${YELLOW}Building for ${name}...${NC}"
    
    # Install target if not present
    rustup target add $target || true
    
    # Build
    cargo build --release --target $target
    
    # Create platform-specific package
    case $target in
        x86_64-unknown-linux-gnu)
            echo -e "${GREEN}Creating Linux AppImage...${NC}"
            create_appimage $target
            ;;
        x86_64-pc-windows-msvc)
            echo -e "${GREEN}Creating Windows installer...${NC}"
            create_windows_package $target
            ;;
        x86_64-apple-darwin|aarch64-apple-darwin)
            echo -e "${GREEN}Creating macOS app bundle...${NC}"
            create_macos_bundle $target
            ;;
    esac
    
    echo -e "${GREEN}✅ ${name} build complete${NC}"
}

# Function to create Linux AppImage
create_appimage() {
    local target=$1
    local appdir="releases/Rusty-$target.AppDir"
    
    # Create AppDir structure
    mkdir -p "$appdir/usr/bin"
    mkdir -p "$appdir/usr/share/applications"
    mkdir -p "$appdir/usr/share/icons/hicolor/256x256/apps"
    
    # Copy binary
    cp "target/$target/release/rusty" "$appdir/usr/bin/"
    
    # Create desktop file
    cat > "$appdir/usr/share/applications/rusty.desktop" << 'EOF'
[Desktop Entry]
Name=Rusty
Exec=rusty
Icon=rusty
Type=Application
Categories=Utility;Archiving;
Comment=Modern ZIP archiver written in Rust
EOF
    
    # Copy icon
    cp "icons/icon.png" "$appdir/usr/share/icons/hicolor/256x256/apps/rusty.png"
    
    # Create AppRun
    cat > "$appdir/AppRun" << 'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
exec "${HERE}/usr/bin/rusty" "$@"
EOF
    chmod +x "$appdir/AppRun"
    
    # Create tarball
    cd releases
    tar -czf "rusty-linux-x86_64.tar.gz" "Rusty-$target.AppDir"
    cd ..
}

# Function to create Windows package
create_windows_package() {
    local target=$1
    local windir="releases/rusty-windows-x86_64"
    
    mkdir -p "$windir"
    
    # Copy binary
    cp "target/$target/release/rusty.exe" "$windir/"
    
    # Create installer script
    cat > "$windir/install.bat" << 'EOF'
@echo off
echo Installing Rusty...
if not exist "%ProgramFiles%\Rusty" mkdir "%ProgramFiles%\Rusty"
copy rusty.exe "%ProgramFiles%\Rusty\rusty.exe"
echo.
echo Installation complete!
echo You can now run 'rusty' from the command line.
pause
EOF
    
    # Create uninstaller
    cat > "$windir/uninstall.bat" << 'EOF'
@echo off
echo Uninstalling Rusty...
if exist "%ProgramFiles%\Rusty\rusty.exe" del "%ProgramFiles%\Rusty\rusty.exe"
if exist "%ProgramFiles%\Rusty" rmdir "%ProgramFiles%\Rusty"
echo Uninstallation complete!
pause
EOF
    
    # Create ZIP package
    cd releases
    zip -r "rusty-windows-x86_64.zip" "rusty-windows-x86_64"
    cd ..
}

# Function to create macOS bundle
create_macos_bundle() {
    local target=$1
    local arch
    
    case $target in
        x86_64-apple-darwin)
            arch="x86_64"
            ;;
        aarch64-apple-darwin)
            arch="aarch64"
            ;;
    esac
    
    local bundle_dir="releases/Rusty-$arch.app"
    
    # Create bundle structure
    mkdir -p "$bundle_dir/Contents/MacOS"
    mkdir -p "$bundle_dir/Contents/Resources"
    
    # Copy binary
    cp "target/$target/release/rusty" "$bundle_dir/Contents/MacOS/"
    
    # Copy icon
    cp "icons/icon.icns" "$bundle_dir/Contents/Resources/"
    
    # Create Info.plist
    cat > "$bundle_dir/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>rusty</string>
    <key>CFBundleIdentifier</key>
    <string>com.rusty.archiver</string>
    <key>CFBundleName</key>
    <string>Rusty</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.14</string>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2024 Rusty. All rights reserved.</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF
    
    # Create DMG
    cd releases
    hdiutil create -volname "Rusty" -srcfolder "Rusty-$arch.app" -ov -format UDZO "rusty-macos-$arch.dmg"
    cd ..
}

# Check if we're on the right platform for certain builds
check_platform() {
    case "$OSTYPE" in
        darwin*)
            echo "🍎 macOS detected"
            CAN_BUILD_MACOS=true
            ;;
        linux*)
            echo "🐧 Linux detected"
            CAN_BUILD_LINUX=true
            ;;
        msys*|cygwin*)
            echo "🪟 Windows detected"
            CAN_BUILD_WINDOWS=true
            ;;
    esac
}

# Main build process
main() {
    check_platform
    
    echo -e "${GREEN}Starting cross-platform build...${NC}"
    
    # Clean previous builds
    cargo clean
    
    # Build for all targets
    build_target "x86_64-unknown-linux-gnu" "Linux x86_64"
    
    # Only build Windows if cross-compilation is set up
    if command -v x86_64-w64-mingw32-gcc &> /dev/null || [[ "$CAN_BUILD_WINDOWS" == true ]]; then
        build_target "x86_64-pc-windows-msvc" "Windows x86_64"
    else
        echo -e "${YELLOW}⚠️  Skipping Windows build (cross-compilation not available)${NC}"
    fi
    
    # Build macOS targets
    if [[ "$OSTYPE" == "darwin"* ]]; then
        build_target "x86_64-apple-darwin" "macOS x86_64"
        build_target "aarch64-apple-darwin" "macOS ARM64"
    else
        echo -e "${YELLOW}⚠️  Skipping macOS builds (requires macOS host)${NC}"
    fi
    
    echo -e "${GREEN}🎉 All builds complete!${NC}"
    echo -e "${GREEN}Release packages created in ./releases/${NC}"
    
    # List created files
    echo -e "${YELLOW}Created packages:${NC}"
    ls -la releases/
}

# Run main function
main "$@"