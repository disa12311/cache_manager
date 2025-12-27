# 🧠 Memory Cache Manager v1.0 (Tauri 2.0)

Advanced Memory Cache Cleaner for Windows with modern Tauri 2.0 UI

## ✨ Features

- **Real Memory Cache Cleaning**: Uses Windows API to actually clear memory cache
- **Modern Tauri 2.0 UI**: Latest framework with improved performance
- **Dual Threshold System**: Start and stop thresholds for smart cleaning
- **Auto-Clean**: Automatic cleaning every 30 seconds when threshold is reached
- **Real-time Monitoring**: Live memory usage display
- **Lightweight**: Small binary size with native performance

## 🚀 Build Instructions (Codespaces/Linux)

### Prerequisites
- Rust 1.70+
- mingw-w64 for cross-compilation

### Quick Build

```bash
# Install mingw-w64 (if not installed)
sudo apt-get update
sudo apt-get install -y mingw-w64

# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Build release
cargo clean
cargo build --release

# File .exe at:
# target/x86_64-pc-windows-gnu/release/memory-cache-manager.exe
```

## 📁 Project Structure

```
memory-cache-manager/
├── .cargo/
│   └── config.toml      # Cross-compile config
├── Cargo.toml           # Tauri 2.0 dependencies
├── build.rs             # Tauri build script
├── tauri.conf.json      # Tauri 2.0 configuration
├── src/
│   ├── main.rs          # Rust backend (Windows API)
│   └── lib.rs           # Library entry
└── ui/
    └── index.html       # Frontend UI
```

## 🎯 What's New in Tauri 2.0

- **windows-rs**: Modern Windows API bindings (replaces winapi)
- **Simplified config**: Cleaner tauri.conf.json structure
- **Better performance**: Improved IPC and rendering
- **Module imports**: ES6 imports in frontend
- **No allowlist needed**: Simplified security model

## 🔧 Configuration

- **Start Threshold**: Memory usage to trigger cleaning (512-8192 MB)
- **Stop Threshold**: Target memory after cleaning (256-4096 MB)
- **Auto-Clean**: Enable/disable automatic cleaning

## ⚠️ Notes

- **Run as Administrator** for best results
- Windows-only (uses Windows API)
- Cleaning process takes 2-10 seconds depending on target
- Safe: Only clears cache, doesn't touch system or application data

## 🛠️ Development

```bash
# Build release
cargo build --release

# Clean build artifacts
cargo clean
```

## 🐛 Troubleshooting

### Error: "failed to find tool"
```bash
sudo apt-get install mingw-w64
```

### Error: "windows crate not found"
```bash
cargo update
cargo build --release
```

### Build takes long
- First build: ~10-15 minutes (compiling Tauri 2.0 + windows-rs)
- Subsequent builds: Much faster with cache

## 📊 Comparison

| Feature | v1.0 (Tauri 2.0) | Previous (Tauri 1.5) |
|---------|------------------|----------------------|
| Windows API | windows-rs 0.52 | winapi 0.3 |
| Config Format | Simplified | Complex allowlist |
| Frontend API | ES6 modules | Global object |
| Performance | Faster IPC | Standard |

## 📝 License

MIT License - Feel free to use and modify

---

**Made with ❤️ using Rust + Tauri 2.0**