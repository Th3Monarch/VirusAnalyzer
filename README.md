# 🛡️ VirusAnalyzer

> **Analyze. Understand. Protect.**

VirusAnalyzer is a **cross-platform desktop application** (Windows, macOS, Linux) for **static malware analysis and threat assessment**.

It combines local file analysis, heuristic detection, hash reputation, optional VirusTotal integration, AI-assisted explanations, quarantine management, system diagnostics, and detailed analysis reports into a single interface.

The project is built with **React + TypeScript + Tauri 2 + Rust**, with a focus on performance, transparency, modularity, and explainable results.

---

## ✨ Features

### 🔍 File Analysis

* Single-file analysis.
* Recursive folder scanning.
* Drag & Drop support.
* SHA-256 / SHA-1 / MD5 hashing.
* Static analysis and heuristic detection.
* Threat scoring from **0–100**.
* Threat levels: 🟢 Clean · 🟡 Low · 🟠 Medium · 🔴 High · 🛑 Critical

### 🧬 Static Analysis

VirusAnalyzer can analyze supported executables and inspect characteristics such as:

* PE structure (Windows).
* Sections, imports, entry point.
* Architecture and entropy.
* Digital signatures.
* Potentially suspicious APIs.
* Other static indicators.

### 🧠 Heuristic Analysis

The analysis engine combines multiple indicators to calculate a threat score.

```text
Suspicious API usage       +25
High entropy               +15
Persistence indicator      +20
Unsigned executable        +10
External reputation        +8
                           ───
                            78/100
```

### 🤖 AI-Assisted Assessment

The AI layer interprets evidence and provides prediction, confidence, explanation, indicators, impact, consequences, recommendations, and attack vectors.

The selected application language is respected by the AI output.

### 🌐 VirusTotal Integration

Optional integration with the VirusTotal API for external reputation via file hashes. Requires an API key. Files are **never** uploaded automatically.

### 🔒 Quarantine

Suspicious files can be isolated with metadata (original path, hash, ID, date, reason). Users can view, restore, or permanently delete quarantined files.

### 💻 Terminal

VirusAnalyzer includes a terminal module adapted to each platform:

| Platform | Shell |
|----------|-------|
| Windows | PowerShell (`powershell.exe`) |
| macOS | System shell (`$SHELL` or `/bin/zsh`) |
| Linux | System shell (`$SHELL` or `/bin/sh`) |

Features include command execution, stdout/stderr, exit codes, duration, cancellation, history, favorites, command reference, and risk classification.

### 📊 Analysis History

Every completed analysis can be stored locally with file info, hashes, timestamps, scores, findings, AI assessment, and reputation data.

### 📄 Reports

Export analysis results as **HTML** or **CSV** with file info, hashes, scores, findings, evidence, reputation, AI assessment, and recommendations.

### 🖥️ Platform Integration

| Feature | Windows | macOS | Linux |
|---------|---------|-------|-------|
| Native notifications | ✅ | ✅ | ✅ |
| System information | ✅ | ✅ | ✅ |
| Terminal integration | PowerShell | POSIX shell | POSIX shell |
| Context menu | ✅ Registry | — | — |
| PE analysis | Full | Partial | Partial |

---

## 🏗️ Architecture

```text
┌──────────────────────────────────────────┐
│              VIRUSANALYZER               │
├──────────────────────────────────────────┤
│                                          │
│              React + TypeScript          │
│                     │                    │
│                     ▼                    │
│                 Tauri 2                  │
│                     │                    │
│                     ▼                    │
│              platform/ layer             │
│       ┌─────────────┼─────────────┐      │
│       ▼             ▼             ▼      │
│   Windows        Linux         macOS     │
│       │             │             │      │
│       ├─────────────┼─────────────┤      │
│       ▼             ▼             ▼      │
│   Scanner       Analyzer      Hashing    │
│       │             │             │      │
│       ├─────────────┼─────────────┤      │
│       ▼             ▼             ▼      │
│   Heuristics    VirusTotal    Quarantine │
│                                          │
└──────────────────────────────────────────┘
```

### Technology Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 19 |
| Language | TypeScript / Rust |
| Styling | Tailwind CSS v4 |
| Build Tool | Vite 7 |
| Desktop Framework | Tauri 2 |
| Hashing | SHA-256 / SHA-1 / MD5 |
| HTTP | ureq (Rust) |
| Serialization | serde / serde_json |
| Platforms | Windows · macOS · Linux |

---

## 📁 Project Structure

```text
VirusAnalyzer/
│
├── src/
│   ├── components/
│   ├── pages/
│   ├── lib/
│   ├── hooks/
│   ├── contexts/
│   │   ├── PlatformContext.tsx        # Platform detection hook
│   │   └── ...
│   ├── types/
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css
│
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── platform/
│   │   │   ├── mod.rs                 # Platform enum + factory
│   │   │   ├── terminal.rs            # TerminalManager trait
│   │   │   ├── windows.rs             # Windows impl
│   │   │   ├── linux.rs               # Linux impl
│   │   │   └── macos.rs               # macOS impl
│   │   ├── scanner/
│   │   ├── analyzer/
│   │   ├── hashing/
│   │   ├── quarantine/
│   │   ├── virustotal/
│   │   ├── rules/
│   │   ├── config/
│   │   ├── models.rs
│   │   ├── powershell/                # Windows-only (#[cfg])
│   │   ├── powershell_reference/      # Windows-only (#[cfg])
│   │   ├── contextmenu/               # Windows-only (#[cfg])
│   │   └── system/
│   │
│   ├── icons/
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── scripts/
│   ├── sign.ps1                       # Windows code signing
│   ├── package.ps1                    # Windows packaging
│   ├── package-linux.sh               # Linux packaging
│   └── package-macos.sh               # macOS packaging
│
├── .github/workflows/
│   ├── ci.yml                         # CI on push/PR
│   └── release.yml                    # Build + release on tags
│
├── package.json
├── README.md
├── LICENSE
└── .gitignore
```

---

## 🚀 Installation

### Requirements

| Platform | Requirements |
|----------|-------------|
| **All** | Node.js 20+, npm, Rust (stable), Tauri CLI |
| **Windows** | Windows 10+, WebView2 Runtime (included in Windows 11) |
| **macOS** | macOS 10.15+, Xcode Command Line Tools |
| **Linux** | `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf` |

### Linux dependencies (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf
```

### Clone and install

```bash
git clone https://github.com/th3monarch/VirusAnalyzer.git
cd VirusAnalyzer
npm install
```

---

## 🧪 Development

```bash
npm run tauri dev
```

The frontend dev server is handled by Vite with hot reload.

---

## 📦 Build

### All platforms

```bash
# Generic build (auto-detects platform)
npm run tauri build
```

### Platform-specific

```bash
# Windows: NSIS installer + portable
npm run build:windows

# Windows: portable only
npm run build:portable

# Linux: .deb + AppImage
npm run build:linux

# macOS: .dmg
npm run build:macos
```

### Build output

```text
dist/
├── VirusAnalyzer-2.0.0-Setup.exe        # Windows NSIS installer
├── VirusAnalyzer-2.0.0.exe              # Windows portable
├── VirusAnalyzer-2.0.0-Portable.zip     # Windows portable ZIP
├── VirusAnalyzer_2.0.0_amd64.deb        # Linux Debian package
├── VirusAnalyzer-2.0.0-amd64.AppImage   # Linux AppImage
├── VirusAnalyzer-2.0.0-aarch64.dmg      # macOS DMG
└── *.sha256                             # SHA-256 checksums
```

---

## ⚙️ Configuration

VirusAnalyzer stores application configuration locally.

Configuration can include: language, theme, VirusTotal API key, context menu settings (Windows only), and other preferences.

API keys and credentials should **never** be committed to Git.

---

## 🔐 Security

VirusAnalyzer is a security analysis tool, so security is a core design consideration:

* Static analysis without executing suspicious files.
* No automatic execution of analyzed files.
* Terminal execution separated from malware analysis.
* Input validation and careful file path handling.
* External reputation services are optional.
* No automatic uploads to third-party services.
* Transparent evidence for threat assessments.

Terminal commands execute with the permissions of the current user. Users should only execute commands they understand and trust.

---

## ⚠️ Disclaimer

VirusAnalyzer is a **malware analysis and threat assessment tool**. It is **not a replacement for a professional endpoint security product or antivirus solution**.

A "Clean" result does **not** guarantee that a file is completely safe. A "High" or "Critical" rating represents an assessment based on available evidence and does not necessarily prove that a file is malware.

The project is intended for: security research, education, malware analysis, system diagnostics, threat assessment, and defensive security experimentation.

Always exercise caution when analyzing unknown files.

---

## 🧠 Design Philosophy

### Analyze
Collect technical evidence from files and the system environment.

### Understand
Explain why an analysis produced a particular result.

### Protect
Provide practical defensive actions such as quarantine and further investigation.

Transparency and explainability are fundamental to the project.

---

## 🌎 Internationalization

* 🇪🇸 Spanish
* 🇺🇸 English

The interface and AI-generated assessments follow the language selected by the user.

---

## 🛠️ Roadmap

* [ ] Advanced PE analysis.
* [ ] More static analysis techniques.
* [ ] Expanded heuristic rule engine.
* [ ] YARA rule support.
* [ ] More file format analysis.
* [ ] Improved reputation analysis.
* [ ] Advanced reporting.
* [ ] SQLite-based persistence.
* [ ] Automated testing suite.
* [ ] Additional languages.

---

## 🤝 Contributing

Contributions, bug reports and suggestions are welcome.

Before submitting a pull request:

1. Test your changes.
2. Make sure the application builds successfully on your platform.
3. Avoid committing secrets or API keys.
4. Keep frontend and backend responsibilities separated.
5. Document significant architectural changes.

For bugs, please include:

* OS and version.
* VirusAnalyzer version.
* Steps to reproduce.
* Expected vs actual behavior.
* Relevant logs or screenshots.

---

## 📜 License

See [LICENSE](LICENSE) for the complete terms.

---

**VirusAnalyzer** · > Analyze. Understand. Protect.

If you find the project useful, consider giving it a ⭐ on GitHub.
