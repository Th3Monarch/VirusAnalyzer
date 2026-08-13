# 🛡️ VirusAnalyzer

> **Analyze. Understand. Protect.**

VirusAnalyzer is a Windows desktop application for **static malware analysis and threat assessment**.

It combines local file analysis, heuristic detection, hash reputation, optional VirusTotal integration, AI-assisted explanations, quarantine management, PowerShell diagnostics, and detailed analysis reports into a single interface.

The project is built with **React + TypeScript + Tauri 2 + Rust**, with a focus on performance, transparency, modularity, and explainable results.

---

## ✨ Features

### 🔍 File Analysis

* Single-file analysis.
* Recursive folder scanning.
* Drag & Drop support.
* SHA-256 hashing.
* SHA-1 hashing.
* MD5 hashing.
* Static analysis.
* Heuristic detection.
* Threat scoring from **0–100**.
* Threat levels:

  * 🟢 Clean
  * 🟡 Low
  * 🟠 Medium
  * 🔴 High
  * 🛑 Critical

### 🧬 Static Analysis

VirusAnalyzer can analyze supported Windows executables and inspect characteristics such as:

* PE structure.
* Sections.
* Imports.
* Entry point.
* Architecture.
* Entropy.
* Digital signatures.
* Potentially suspicious APIs.
* Other static indicators.

The goal is to provide **evidence and context**, rather than simply returning a binary "virus / not virus" result.

### 🧠 Heuristic Analysis

The analysis engine combines multiple indicators to calculate a threat score.

For example:

```text
Suspicious API usage       +25
High entropy               +15
Persistence indicator      +20
Unsigned executable        +10
External reputation        +8
                           ───
                            78/100
```

Every score should be explainable through the findings that contributed to it.

### 🤖 AI-Assisted Assessment

The AI layer interprets the evidence collected by the analysis engine and provides:

* Prediction.
* Confidence.
* Explanation.
* Indicators.
* Potential impact.
* System consequences.
* Recommended actions.
* Potential attack vector.

The AI is designed as an **explanatory analysis layer**, not as the sole malware detection mechanism.

The selected application language is respected by the AI output.

### 🌐 VirusTotal Integration

VirusAnalyzer can optionally integrate with the VirusTotal API.

When configured, the application can use file hashes to obtain external reputation information.

VirusTotal integration is optional and requires an API key.

Files should not be uploaded to external services automatically without explicit user consent.

### 🔒 Quarantine

Suspicious files can be isolated from their original location.

The quarantine system stores metadata such as:

* Original path.
* File hash.
* Quarantine ID.
* Date.
* Reason for quarantine.

Users can:

* View quarantined files.
* Restore files.
* Permanently delete files.

### 💻 PowerShell

VirusAnalyzer includes an advanced PowerShell module for Windows diagnostics and administration.

Features include:

* PowerShell command execution.
* Standard output.
* Error output.
* Exit codes.
* Execution duration.
* Command cancellation.
* Command history.
* Favorites.
* Command reference.
* Command risk classification.

PowerShell commands execute with the permissions of the current Windows user.

**VirusAnalyzer does not automatically execute PowerShell commands during malware analysis.**

### 📊 Analysis History

Every completed analysis can be stored in the local history.

History includes information such as:

* File name.
* Path.
* Hashes.
* Timestamp.
* Threat score.
* Threat level.
* Findings.
* Analysis information.
* AI assessment.
* External reputation when available.

### 📄 Reports

Analysis results can be exported as:

* HTML.
* CSV.

Reports can contain:

* File information.
* Hashes.
* Threat score.
* Threat level.
* Findings.
* Evidence.
* Reputation information.
* AI assessment.
* Recommendations.

### 🖥️ Windows Integration

VirusAnalyzer supports Windows-specific functionality including:

* Native notifications.
* Windows context-menu integration.
* System information.
* PowerShell integration.

---

# 🏗️ Architecture

VirusAnalyzer uses a hybrid desktop architecture.

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
│                  Rust                    │
│                     │                    │
│       ┌─────────────┼─────────────┐      │
│       ▼             ▼             ▼      │
│    Scanner       Analyzer      Hashing   │
│       │             │             │      │
│       ├─────────────┼─────────────┤      │
│       ▼             ▼             ▼      │
│   Heuristics    VirusTotal    Quarantine │
│                                          │
└──────────────────────────────────────────┘
```

## Technology Stack

| Layer               | Technology            |
| ------------------- | --------------------- |
| Frontend            | React                 |
| Language            | TypeScript            |
| Styling             | Tailwind CSS          |
| Build Tool          | Vite                  |
| Desktop Framework   | Tauri 2               |
| Backend             | Rust                  |
| HTTP                | reqwest               |
| Serialization       | serde / serde_json    |
| Hashing             | SHA-256 / SHA-1 / MD5 |
| Platform            | Windows               |
| External Reputation | VirusTotal API        |

---

# 📁 Project Structure

```text
VirusAnalyzer/
│
├── src/
│   ├── components/
│   ├── pages/
│   ├── lib/
│   ├── hooks/
│   ├── contexts/
│   ├── types/
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css
│
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── scanner/
│   │   ├── analyzer/
│   │   ├── hashing/
│   │   ├── quarantine/
│   │   ├── virustotal/
│   │   ├── rules/
│   │   ├── config/
│   │   ├── powershell/
│   │   └── system/
│   │
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── public/
├── package.json
├── README.md
├── LICENSE
└── .gitignore
```

---

# 🚀 Installation

## Requirements

Before building VirusAnalyzer, make sure you have the required development environment installed:

* Windows 10/11.
* Node.js.
* npm.
* Rust.
* Tauri prerequisites.

Then clone the repository:

```bash
git clone https://github.com/YOUR_USERNAME/VirusAnalyzer.git
cd VirusAnalyzer
```

Install frontend dependencies:

```bash
npm install
```

---

# 🧪 Development

Run the application in development mode:

```bash
npm run tauri dev
```

The frontend development server is handled by Vite.

---

# 📦 Build

Build the production application:

```bash
npm run tauri build
```

The generated Windows binaries and installers will be placed in the Tauri build output directory.

---

# ⚙️ Configuration

VirusAnalyzer stores application configuration locally.

Configuration can include:

```text
Language
Theme
VirusTotal API key
Context menu settings
Other application preferences
```

API keys and credentials should **never be committed to Git**.

Use environment/configuration examples when sharing development configuration.

---

# 🔐 Security

VirusAnalyzer is a security analysis tool, so security is a core design consideration.

The application is designed to:

* Perform static analysis without executing suspicious files.
* Avoid automatically executing analyzed files.
* Keep PowerShell execution separate from malware analysis.
* Validate user input.
* Handle file paths carefully.
* Avoid unnecessary privileges.
* Keep external reputation services optional.
* Avoid automatically uploading files to third-party services.
* Provide transparent evidence for threat assessments.

PowerShell commands execute with the permissions of the current Windows user.

Users should only execute commands they understand and trust.

---

# ⚠️ Disclaimer

VirusAnalyzer is a **malware analysis and threat assessment tool**.

It is **not a replacement for a professional endpoint security product or antivirus solution**.

A result such as:

```text
Clean
```

does **not** guarantee that a file is completely safe.

Likewise, a:

```text
High
Critical
```

rating represents an assessment based on the available evidence and does not necessarily prove that a file is malware.

The project is intended for:

* Security research.
* Education.
* Malware analysis.
* System diagnostics.
* Threat assessment.
* Defensive security experimentation.

Always exercise caution when analyzing unknown files.

---

# 🧠 Design Philosophy

VirusAnalyzer follows three principles:

### Analyze

Collect technical evidence from files and the Windows environment.

### Understand

Explain why an analysis produced a particular result.

### Protect

Provide practical defensive actions such as quarantine and further investigation.

The objective is not simply:

```text
"This file is a virus."
```

Instead:

```text
"This file received a threat score of 78/100
because these specific indicators were detected."
```

Transparency and explainability are fundamental to the project.

---

# 🌎 Internationalization

VirusAnalyzer supports multiple languages.

Currently supported:

* 🇪🇸 Spanish
* 🇺🇸 English

The interface and AI-generated assessments are designed to follow the language selected by the user.

Technical identifiers such as:

```text
SHA-256
KERNEL32.dll
CreateProcessW
PowerShell
```

remain in their conventional technical form when appropriate.

---

# 🛠️ Roadmap

Future development may include:

* [ ] Advanced PE analysis.
* [ ] More static analysis techniques.
* [ ] Expanded heuristic rule engine.
* [ ] YARA rule support.
* [ ] More file format analysis.
* [ ] Improved reputation analysis.
* [ ] Enhanced threat scoring.
* [ ] More detailed analysis timelines.
* [ ] Advanced reporting.
* [ ] SQLite-based persistence.
* [ ] Automated testing suite.
* [ ] Improved Windows integration.
* [ ] Additional languages.

The roadmap may change as the project evolves.

---

# 🤝 Contributing

Contributions, bug reports and suggestions are welcome.

Before submitting a pull request:

1. Test your changes.
2. Make sure the application builds successfully.
3. Avoid committing secrets or API keys.
4. Keep frontend and backend responsibilities separated.
5. Document significant architectural changes.

For bugs, please include:

* Windows version.
* VirusAnalyzer version.
* Steps to reproduce.
* Expected behavior.
* Actual behavior.
* Relevant logs or screenshots.

---

# 📜 License

This project is distributed under the license included in this repository.

See:

```text
LICENSE
```

for the complete terms.

---

# ⭐ Project

**VirusAnalyzer**

> Analyze. Understand. Protect.

A Windows-focused malware analysis and threat assessment application built with **React, TypeScript, Tauri 2 and Rust**.

If you find the project useful, consider giving it a ⭐ on GitHub.
