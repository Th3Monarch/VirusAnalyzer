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

<<<<<<< HEAD
## Comandos

```bash
# Instalar dependencias
npm install

# Desarrollo (con hot reload de Tauri + Vite)
npm run tauri dev

# Compilar frontend (verificación TS)
npm run build

# Distribución completa de Windows (Setup + exe + portable + SHA-256 en dist/)
npm run build:windows

# Solo la distribución portable (exe + zip + SHA-256 en dist/)
npm run build:portable
```

## Descargas

Tras ejecutar `npm run build:windows`, la carpeta `dist/` contiene las tres
distribuciones y sus checksums SHA-256:

```text
dist/
├── VirusAnalyzer-2.0.0-Setup.exe
├── VirusAnalyzer-2.0.0.exe
├── VirusAnalyzer-2.0.0-Portable.zip
├── VirusAnalyzer-2.0.0-Setup.exe.sha256
├── VirusAnalyzer-2.0.0.exe.sha256
└── VirusAnalyzer-2.0.0-Portable.zip.sha256
```

### Instalador (`VirusAnalyzer-2.0.0-Setup.exe`)

**Opción recomendada para la mayoría de usuarios.** Instalador NSIS que
instala VirusAnalyzer para el usuario actual (`%LOCALAPPDATA%\VirusAnalyzer`),
crea accesos directos en el menú Inicio y en el escritorio, registra la
aplicación en "Agregar o quitar programas", incluye desinstalador y usa el
icono oficial. La versión se lee de la configuración del proyecto.

### Portable (`VirusAnalyzer-2.0.0-Portable.zip`)

Para usuarios que no quieren instalar la aplicación: se extrae en cualquier
carpeta (`C:\Apps\VirusAnalyzer\`, `D:\Portable\VirusAnalyzer\`, un USB…)
y se ejecuta directamente `VirusAnalyzer.exe`. No depende de rutas del equipo
de desarrollo. El frontend va embebido en el ejecutable, por lo que el ZIP
contiene únicamente el binario real de Tauri. Requiere el **WebView2 Runtime**
de Windows (incluido en Windows 11 y en la mayoría de Windows 10). La
configuración y el historial se guardan en la carpeta de datos de la
aplicación (`%APPDATA%\com.virusanalyzer.desktop`).

### Ejecutable (`VirusAnalyzer-2.0.0.exe`)

Distribución avanzada/manual: el binario release real generado por
Rust/Tauri, sin instalador. Útil para copias puntuales, pero comparte los
requisitos del portable (WebView2 Runtime) y no crea accesos ni registro.

### Verificación de integridad

```bash
sha256sum -c VirusAnalyzer-2.0.0-Setup.exe.sha256
# o en Windows:
certutil -hashfile VirusAnalyzer-2.0.0-Setup.exe SHA256
```

### Firma de código

Los binarios **no están firmados** (`Code signing status: Unsigned`): Windows
Defender/SmartScreen pueden mostrar advertencias al ejecutarlos, y la ausencia
de firma es un factor que los motores heurísticos (incluido el `!ml` de
Microsoft) suelen considerar sospechoso en binarios recién compilados.

No se utilizan certificados autofirmados ni se simula una firma de confianza.
El proyecto está **preparado para firmar con un certificado Authenticode
legítimo** sin almacenar ningún secreto en el repositorio: el build solo firma
cuando las credenciales están disponibles en el entorno.

`npm run build:windows` ejecuta Tauri con `bundle.windows.signCommand`
(`tauri.conf.json`), que invoca `scripts/sign.ps1` para cada binario firmable:
el ejecutable principal, los plugins NSIS, el instalador y el desinstalador
embebido. El script lee toda la configuración de variables de entorno:

| Variable | Uso |
| --- | --- |
| `VA_SIGN_THUMBPRINT` | Huella del certificado instalado en el almacén de Windows. **No requiere contraseña** (opción recomendada). |
| `VA_SIGN_PFX` | Ruta a un archivo `.pfx` (usar junto a `VA_SIGN_PASSWORD`). |
| `VA_SIGN_PASSWORD` | Contraseña del `.pfx`. Nunca se registra ni se guarda. |
| `VA_SIGN_TIMESTAMP_URL` | *(opcional)* Servidor RFC3161. Por defecto `http://timestamp.digicert.com`. |
| `VA_SIGN_SIGNTOOL` | *(opcional)* Ruta explícita a `signtool.exe`. Si no se define, se busca en el PATH y en el Windows SDK. |

Si no se define `VA_SIGN_THUMBPRINT` ni `VA_SIGN_PFX`, la firma se omite y el
build continúa (los binarios quedan sin firmar).

**Preparación (certificado en el almacén, sin contraseñas):**

```powershell
# Importa tu .pfx al almacén del usuario (pedirá la contraseña una vez).
Import-PfxCertificate -FilePath "C:\certs\mi-cert.pfx" -CertStoreLocation Cert:\CurrentUser\My

# Copia la huella (Thumbprint) del certificado.
Get-ChildItem Cert:\CurrentUser\My | Format-Table Subject, Thumbprint

# Guárdala como variable de entorno (persistente):
setx VA_SIGN_THUMBPRINT "A1B2C3D4..."
# o solo para el build actual:
$env:VA_SIGN_THUMBPRINT = "A1B2C3D4..."
```

**Alternativa con `.pfx`:**

```powershell
$env:VA_SIGN_PFX = "C:\certs\mi-cert.pfx"
$env:VA_SIGN_PASSWORD = "MiContraseña"
```

**Verificación tras el build:**

```powershell
Get-AuthenticodeSignature "dist\VirusAnalyzer-2.0.0-Setup.exe" | Format-List Status, StatusMessage, SignerCertificate
# y para ver la cadena completa:
signtool verify /pa /v "dist\VirusAnalyzer-2.0.0-Setup.exe"
```

Alternativa equivalente sin `scripts/sign.ps1`: fijar `bundle.windows.certificateThumbprint` en `tauri.conf.json`, que usa la configuración incorporada de Tauri con `signtool` (requiere que el certificado esté en el almacén y fallará el build si no lo está).

## Detecciones antivirus

VirusAnalyzer es una aplicación defensiva de análisis de malware. Algunos
motores antivirus pueden marcar aplicaciones de Windows recién compiladas o sin
firmar. Si VirusAnalyzer es detectado:

- **Verifica el hash SHA-256** de la distribución contra los `.sha256` de
  `dist/` antes de asumir que el archivo es malicioso.
- **Revisa la detección**: la detección `Trojan:Win32/Wacatac.B!ml` (sufijo
  `!ml`) corresponde a la heurística de aprendizaje automático de Microsoft,
  no a una firma de un comportamiento específico.
- **Revisa el código fuente**: el proyecto no utiliza ofuscación, packing,
  anti-análisis ni técnicas de evasión de antivirus. Todo el comportamiento
  (escaneo estático, consulta a VirusTotal por hash solo si se habilita,
  cuarentena manual, PowerShell bajo demanda del usuario, menú contextual
  opcional) está documentado en este README.

El proyecto se distribuye sin firma de código; obtener un certificado de firma
de código legítimo y firmar los releases es la mitigación más eficaz para la
mayoría de estas advertencias.

## Estructura
=======
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
>>>>>>> 65d7dcfd32c9f3549e1d763d0be4200b23834fbf

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
├── scripts/
│   └── package.ps1           # ensambla dist/ (setup, exe, portable, SHA-256)
├── dist/                     # distribuciones finales (generado)
├── dist-app/                 # build del frontend para Tauri (generado)
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
