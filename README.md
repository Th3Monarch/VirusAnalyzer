# VirusAnalyzer 2.0

> **Analyze. Understand. Protect.**

Herramienta de escritorio para Windows que realiza **análisis estático** de
archivos sospechosos, genera evidencia técnica, una puntuación de riesgo
explicable y una evaluación asistida por IA basada únicamente en esa evidencia.

VirusAnalyzer **no** es un antivirus comercial ni afirma detectar todo malware.

---

## Estado del proyecto

### FASE 1 — FOUNDATION ✅

- Proyecto Tauri 2 + React 19 + TypeScript + Vite 7 + Tailwind CSS v4.
- Backend Rust modular (`src-tauri/src/`).
- Navegación completa: Dashboard, Scan, Results, Analysis, Quarantine, Rules,
  System, PowerShell y Settings.
- Layout con panel lateral, temas claro/oscuro y animaciones sutiles.
- Internacionalización ES/EN centralizada (`t("clave.punto")`).
- Sistema de configuración en JSON con versionado de esquema (migración
  futura a SQLite).
- Comandos Tauri: `get_config`, `save_config`, `get_app_info`,
  `get_system_info`.
- Compila sin errores: `tsc` + `vite build` y `cargo build` (release OK).

### FASE 4 — HEURISTIC ENGINE ✅

- **Catálogo de 28 reglas** (`src-tauri/src/rules/mod.rs`) organizado por
  categorías (process, persistence, powershell, packing, network,
  signatures, general), cada una con id, nombre, descripción, severidad y
  puntos.
- Detección por **imports de API** (inyección de procesos, RWX, keylogging,
  anti-debug, persistencia, servicios, descarga de archivos, WinHTTP/WinINet,
  sockets, DNS), por **secciones PE** (UPX/packers, entropía de `.text`,
  secciones writable+exec, número de secciones) y por **entropía global**.
- **Análisis de strings** (`analyzer/keywords.rs`): palabras clave
  sospechosas (powershell, schtasks, EICAR…) aplicadas **solo a contenido
  no ejecutable** para evitar ruido en binarios con datos embebidos.
- Detección de **desajuste tipo/extensión** (p. ej. PE disfrazado de .pdf),
  con lista blanca de alias legítimos (docx/zip, exe/dll, jpg/jpeg…).
- **Scoring**: suma ponderada de puntos con tope 100 y niveles
  Clean(0) · Low(1–14) · Medium(15–34) · High(35–64) · Critical(65+).
- Lista de firmas hash conocidas (vacía por defecto, extensible en
  `KNOWN_HASHES`).
- `ScanResult.findings`, `threatScore` y `threatLevel` reales; comando
  `get_rules` para la página de Reglas.
- UI: barra de puntuación en Analysis, lista de hallazgos con severidad/
  categoría/evidencia/puntos, y página Reglas con el catálogo cargado del
  backend.
- Pruebas unitarias del motor (inyección, persistencia, keywords en scripts,
  scoring/niveles, binario propio).

### FASE 5 — VIRUSTOTAL (REPUTACIÓN POR HASH) ✅

- **Consulta opcional por hash** (`src-tauri/src/virustotal/mod.rs`): solo se
  envía a VirusTotal el MD5/SHA-1/SHA-256 del archivo; **el contenido nunca
  se sube**.
- **Consentimiento explícito**: la integración está desactivada por defecto y
  solo se activa con el toggle de Ajustes (`virustotalEnabled`); la API key
  se guarda en la configuración y nunca se registra.
- Cliente HTTP `ureq` con timeout de 20 s: `GET /api/v3/files/{hash}` con
  cabecera `x-apikey`. Manejo de 404 (hash no reportado → disponible sin
  error), 401/403 (clave), 429 (límite) y errores de red/JSON.
- Parseo de `last_analysis_stats` (malicious/suspicious/harmless/undetected/
  timeout/type-unsupported), `reputation`, `times_submitted`, fechas
  (epoch → ISO), `meaningful_name`, `magic`, `size` y tabla de motores con
  amenazas detectadas (los "malicious/suspicious" se listan como nombres de
  amenaza). Permalink a `virustotal.com/gui/file/{hash}/detection`.
- Integrado en `file_scan` (solo archivo individual, solo si la key existe y
  la integración está activa): el resultado se guarda en
  `ScanResult.reputation` y la línea temporal registra cada etapa.
- Comando `virustotal_lookup(hash)` para consulta manual desde Analysis
  (valida habilitación, clave y longitud de hash 32/40/64).
- UI: toggle de consentimiento y aviso en Settings, estado en Dashboard, y
  tarjeta de reputación en Analysis (conteos, amenazas, motores, "Ver en
  VirusTotal" con `plugin-opener` y botón de comprobación manual por hash).
- **Feedback explícito en la comprobación manual**: si VirusTotal no está
  habilitado o falta la clave, el botón avisa de inmediato (sin esperar al
  backend) con un aviso visible y acceso directo a Ajustes; los errores de
  red/clave/límite también se muestran en un aviso destacado, no como texto
  discreto.
- Pruebas unitarias: parseo de respuesta real, 404 → no disponible y
  errores de cuota.

### FASE 6 — AI ASSESSMENT (EVALUACIÓN BASADA EN EVIDENCIA) ✅

- **Motor local y determinista** (`src-tauri/src/assessment/mod.rs`), sin red y
  sin API de IA: sintetiza hallazgos heurísticos, análisis estático y
  reputación de VirusTotal en un informe en lenguaje natural.
- **Nunca inventa resultados**: cada afirmación procede de datos reales ya
  extraídos (evidencia de reglas, tipo/entropía/PE, conteos de VirusTotal).
- **Veredicto** derivado del nivel + reputación externa (clean /
  likely_clean / suspicious / malicious); un hash flaggeado por 2+ motores
  externos asciende el veredicto aunque el score local sea bajo.
- **Confianza** (0–100 %) calculada por fuerza de la evidencia: acuerdo entre
  motor local y VirusTotal la refuerza; hash no reportado la reduce.
- **Resumen** de una o dos frases con el nombre del archivo y los conteos.
- **Explicación** por párrafos: línea base (tipo, entropía, PE, firma,
  reputación) + un párrafo por categoría con hallazgos y su evidencia.
- **Indicadores** concretos por hallazgo, **impacto potencial**,
  **consecuencias de sistema**, **acciones recomendadas** y **vectores de
  ataque** generados a partir de las categorías que realmente dispararon.
- Integrado en `file_scan` (`ScanResult.ai_assessment`) con entrada en la
  línea temporal.
- UI: tarjeta "Evaluación de la IA" en Analysis con veredicto localizado,
  confianza, categorías clave, explicación plegable y detalle expandible.
- **Idioma del informe (corrección)**: el motor genera el contenido
  **directamente en el idioma seleccionado** (`es`/`en`), no traduce texto
  producido en inglés. Cada idioma tiene su propio catálogo de plantillas
  (`Lang` ES/EN) para resumen, explicación, impacto, consecuencias, acciones y
  vectores; las descripciones de reglas tienen ficha en español
  (`rules::description_in`). El idioma viaja de la UI a la config persistida
  y se lee al iniciar cada escaneo (`ScanContext.language` → `assessment::build`)
  por lo que el cambio en tiempo real se aplica al siguiente análisis; un
  valor desconocido cae en `en` solo como fallback técnico (un `es` explícito
  siempre produce español). Cada `ScanResult` almacena `language` para que el
  historial conserve su idioma. Los términos técnicos (APIs, hashes, nombres y
  categorías de reglas) se conservan originales en ambos idiomas. Una
  validación ligera comprueba la lengua de salida y registra desviaciones
  (`AI LANGUAGE VALIDATION FAILED`).
- Pruebas unitarias: veredicto limpio/malicioso, refuerzo de confianza por
  VirusTotal, escalado del veredicto por flags externos, salida en español
  (resumen, explicación, categorías, reglas e impacto) y en inglés, y fallback
  de idioma para valores desconocidos.
- **Historial persistente (corrección)**: el historial y los resultados
  completos se guardan en `history.json` dentro del directorio de configuración
  (`app_config_dir`), se cargan al arrancar y se persisten tras cada escaneo,
  por lo que los análisis sobreviven al reinicio. El id estable (UUID) que ve
  la UI en la lista es **la misma clave** con la que se guardan los resultados
  (`guard.results.insert(entry.id, …)`), eliminando el desajuste que hacía que
  "Más detalles" no encontrara el análisis (`get_scan_result`/`get_analysis_by_id`
  ahora localizan el resultado por el id del historial; también arregla el
  informe HTML/CSV y la vista previa). Las entradas antiguas sin `id` reciben
  un id estable derivado de datos inmutables (sha256 de ruta + nombre + fecha +
  tipo, prefijo `legacy-`) que se persiste una sola vez y no se regenera en
  cada carga. La UI distingue explícitamente `loading` / `success` /
  `notFound` / `error` y consulta al backend por id (sin acceder al historial
  por índice).

### FASE 9 — POLISH (UX, ACCESIBILIDAD, RENDIMIENTO) ✅

- **Notificaciones (toasts)**: sistema ligero (`src/contexts/ToastContext.tsx`)
  con tipos éxito/error/info, auto-descarte, región `aria-live` y botón de
  cierre; sustituye a los `window.alert` de análisis, cuarentena e informes.
- **Accesibilidad**:
  - `aria-label` en botones de solo icono (idioma/tema, copiar hash).
  - `role="status"` / `aria-live="polite"` en avisos de escaneo, cuarentena
    e informes; `role="alert"` en errores de página.
  - Modal de vista previa del informe como `role="dialog"` con `aria-modal`,
    cierre con `Escape`, foco al botón de cerrar y restauración del foco.
- **Animaciones** (`src/index.css`): transición de página existente +
  animaciones de aparición para modales (`va-pop`) y toasts (`va-toast`),
  micro-interacción de presión en botones y **soporte de
  `prefers-reduced-motion`** que anula animaciones y transiciones.
- **Rendimiento**:
  - Carga diferida de páginas con `React.lazy` + `Suspense` (code splitting):
    el bundle principal baja de ~361 KB a ~277 KB y cada página se carga bajo
    demanda.
  - `React.memo` en `LevelBadge`, `SeverityBadge` y `StatCard` (renderizados
    en listas).
  - `useDeferredValue` en la búsqueda de Results para mantener la UI fluida.

### FASE 8 — REPORTING (HTML / CSV) ✅

- **Informes autocontenidos** (`src-tauri/src/report/mod.rs`): generados a
  partir de los resultados ya almacenados en memoria; el módulo **nunca**
  vuelve a escanear ni consulta VirusTotal.
- **HTML**: documento autocontenido (CSS embebido, sin scripts ni recursos
  externos) con secciones de resumen, evaluación de la IA (veredicto,
  confianza, explicación), hallazgos, análisis estático, reputación de
  VirusTotal, hashes y línea temporal. Todo dato de archivos/usuario se
  escapa para evitar inyección de HTML.
- **CSV** (RFC 4180, campos con comas/comillas escapados): informe de archivo
  en una fila (id, hashes, score, nivel, veredicto, confianza, hallazgos,
  evidencia, conteos de VirusTotal) e informe de carpeta en dos secciones
  (resumen + listado de archivos con hashes y errores).
- Admite tanto análisis de **archivo** como de **carpeta** (detección
  automática por el contenido del resultado).
- Comandos Tauri: `export_report` (escribe el archivo) y `preview_report`
  (devuelve el contenido para vista previa).
- UI: botones **Vista previa / Exportar HTML / Exportar CSV** en la página de
  análisis (diálogo de guardado nativo con extensión por defecto) y **vista
  previa** en un modal (HTML renderizado en iframe sandbox, CSV como texto).
- Pruebas unitarias: escape de nombres peligrosos, secciones del HTML,
  escape CSV y despacho por tipo de resultado.

### FASE 7 — QUARANTINE (AISLAR / RESTAURAR / ELIMINAR) ✅

- **Aislar** (`src-tauri/src/quarantine/mod.rs`): mueve el archivo (no lo
  copia) a un directorio de cuarentena —el configurado por el usuario en
  Ajustes o la carpeta de datos de la app— con nombre seguro `Q-<año>-<secuencia>`
  y registra la metadata en un manifiesto `manifest.json` (ruta original,
  nombre, hashes, tamaño, motivo, nivel y fecha).
- **Restaurar**: devuelve el archivo a su ubicación original recreando los
  directorios si es necesario. **No sobrescribe** rutas existentes: si la
  ubicación original ya está ocupada, rechaza la operación para no perder
  datos.
- **Eliminar definitivamente**: borra el blob y su registro del manifiesto.
- **Regla de seguridad**: nunca se aísla ni se elimina un archivo
  automáticamente por tener una puntuación alta; siempre es una acción
  explícita del usuario (con confirmación en la UI).
- Comandos Tauri: `quarantine_file`, `get_quarantine` (directorio + entradas),
  `restore_quarantined`, `delete_quarantined`.
- UI: página **Cuarentena** real (tabla con ID, archivo, ruta original, nivel,
  tamaño, fecha, motivo y acciones Restaurar/Eliminar, directorio efectivo,
  estados de carga/error), botón **Aislar** en el análisis de archivo y
  configuración de la **carpeta de cuarentena** en Ajustes (selector de
  carpeta y restablecimiento al valor por defecto).
- Pruebas unitarias: mover/restaurar ida y vuelta, rechazo de sobrescritura,
  borrado y secuenciación de IDs.

### FASE 3 — STATIC ANALYSIS ✅

- Detección de tipo por **magic bytes** (crate `infer`) con fallback por
  extensión (scripts, documentos…).
- **Entropía de Shannon** global (streaming) y por sección PE.
- **Parser PE propio** (`src-tauri/src/analyzer/pe.rs`, sin dependencias de
  formato): cabeceras DOS/NT, arquitectura (x86/x64/arm…), subsistema,
  punto de entrada, base de imagen, marca de tiempo, secciones con flags y
  entropía, **imports** (DLL + funciones), **exports** y detección de
  **certificado Authenticode**.
- Integrado en el escaneo de archivo individual: `ScanResult.staticAnalysis`
  se rellena y la línea temporal refleja cada etapa.
- UI: tarjeta de análisis estático en Analysis (tipo, entropía con barra,
  resumen PE, tabla de secciones, imports expandibles, exports).
- Prueba unitaria que valida el parser contra el propio binario de test.

### FASE 2 — FILE SCANNER ✅

- Hashing streaming de MD5, SHA-1 y SHA-256 (`src-tauri/src/hashing/mod.rs`),
  con las tres opciones configurables desde `AppConfig.scan`.
- Escaneo de archivo individual y de carpeta recursiva en un hilo aparte,
  con **progreso por eventos**, **cancelación** y respeto de
  `maxFileSizeMb` / `followSymlinks`.
- Historial de análisis **en memoria** (`ScanStore`) con resultados
  completos indexados por id.
- Comandos Tauri: `scan_path`, `cancel_scan`, `get_scan_history`,
  `get_scan_result`, `get_path_info`.
- Eventos: `scan-progress`, `scan-completed`, `scan-error`, `scan-cancelled`.
- Frontend: selección de archivo/carpeta (plugin dialog), **drag & drop**
  nativo, barra de progreso, cancelación, página Results con historial
  real y búsqueda, página Analysis con hashes (copiar), timeline y resumen
  de carpetas. Dashboard muestra el historial real.
- El análisis heurístico (findings/scoring) llega en FASE 4; hasta entonces
  todo se marcaba `Clean` sin inventar resultados.

### Roadmap

| Fase | Contenido | Estado |
| ---- | --------- | ------ |
| 1 | Foundation (navegación, temas, idiomas, configuración) | ✅ |
| 2 | File Scanner (archivos/carpetas, drag & drop, hashes) | ✅ |
| 3 | Static Analysis (PE, entropía, imports, secciones, firmas) | ✅ |
| 4 | Heuristic Engine (reglas, findings, scoring, niveles) | ✅ |
| 5 | VirusTotal (opcional, por hash, consentimiento explícito) | ✅ |
| 6 | AI Assessment (explicación basada en evidencia) | ✅ |
| 7 | Quarantine (aislar / restaurar / eliminar) | ✅ |
| 8 | Reporting (HTML / CSV) | ✅ |
| 9 | Polish (UX, animaciones, accesibilidad, rendimiento) | ✅ |

---

## Requisitos

- Windows 10/11 con WebView2 Runtime.
- Node.js 20+ (instalado en `M:\DevTools\nodejs` en este entorno).
- Rust stable (MSVC toolchain) — `M:\DevTools\rustup` / `M:\DevTools\cargo`.
- Microsoft Visual C++ Build Tools (con la carga de trabajo VCTools) —
  `M:\DevTools\VSBuildTools`.

## Comandos

```bash
# Instalar dependencias
npm install

# Desarrollo (con hot reload de Tauri + Vite)
npm run tauri dev

# Compilar frontend (verificación TS)
npm run build

# Build de producción (instalador en src-tauri/target/release/bundle)
npm run tauri build
```

## Estructura

```
├── src/                      # Frontend React
│   ├── components/           # layout/ y ui/
│   ├── pages/                # una página por sección
│   ├── lib/                  # i18n, tauri, defaults, format
│   ├── hooks/
│   ├── types/                # tipos base compartidos
│   ├── contexts/             # Config, Theme, Language
│   ├── App.tsx               # rutas
│   ├── main.tsx
│   └── index.css             # tema Tailwind (claro/oscuro)
│
├── src-tauri/                # Backend Rust
│   ├── src/
│   │   ├── lib.rs            # comandos Tauri
│   │   ├── models.rs         # tipos compartidos (ScanResult, Finding...)
│   │   ├── config/           # configuración JSON + migraciones
│   │   ├── system/           # información del sistema
│   │   ├── scanner/          # escaneo de archivos/carpetas + historial
│   │   ├── analyzer/         # análisis estático (tipo, entropía, PE, keywords)
│   │   ├── hashing/          # MD5, SHA-1, SHA-256 (streaming)
│   │   ├── rules/            # reglas heurísticas: catálogo, evaluate, scoring
│   │   ├── assessment/       # evaluación explicativa basada en evidencia (FASE 6)
│   │   ├── virustotal/       # reputación por hash (FASE 5)
│   │   ├── quarantine/       # aislar / restaurar / eliminar (FASE 7)
│   │   └── report/           # informes HTML / CSV (FASE 8)
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── public/
├── package.json
└── .gitignore
```

## Flujo Frontend → Tauri → Rust

1. El frontend llama `invoke("nombre_comando", { args })` (ver `src/lib/tauri.ts`).
2. Tauri enruta al comando Rust correspondiente (`src-tauri/src/lib.rs`).
3. Rust accede al sistema (archivos, hashes, red) y devuelve datos
   serializados en `camelCase` con `serde`.
4. El frontend actualiza la UI; los errores se capturan y muestran de forma
   controlada.

## Seguridad

- Análisis **estático**: nunca se ejecutan los archivos analizados.
- VirusTotal es opcional y se consulta por hash; nunca se suben archivos sin
  consentimiento explícito.
- La API key de VirusTotal se guarda en la configuración y nunca se registra.
- PowerShell se trata como función administrativa avanzada, separada del
  análisis y sin ejecución automática. El ejecutor (`src-tauri/src/powershell.rs`)
  arranca `powershell.exe` con los permisos del usuario, con timeout de 30 s,
  cancelación explícita y una única ejecución a la vez; nunca usa `cmd.exe` ni
  eleva privilegios. Los comandos de alto riesgo (`powershell_reference.rs`)
  exigen confirmación explícita en la interfaz. El historial local
  (`va:ps:history`, `va:ps:favorites`) filtra comandos que parecen contener
  secretos (passwords, tokens, claves…). La referencia (`/ps-reference`) es un
  catálogo educativo localizado (26 comandos en 7 categorías): cargar un comando
  rellena la terminal pero nunca lo ejecuta.
- Validación de rutas y entradas; errores controlados en ambos extremos.

## Menú contextual de Windows

- **Integración con el Explorador** (`src-tauri/src/contextmenu.rs`): registra
  «Analizar con VirusAnalyzer» en `HKCU\Software\Classes\*\shell\VirusAnalyzer`
  usando `reg.exe` (binario del sistema, sin pasar por un shell). Solo afecta
  al **usuario actual** y no requiere privilegios de administrador.
- Se aplica a **archivos y carpetas** (`*`); incluye el icono de la app y
  ejecuta `"<ruta del ejecutable>" "%1"`.
- **Toggle en Ajustes** (estado real consultado al abrir la página): activar/
  desactivar con feedback por toast; la preferencia se refleja en
  `contextMenuEnabled` de la configuración.
- **Análisis directo al lanzar**: el menú abre la aplicación pasando la ruta
  como argumento; `take_launch_path` la entrega una sola vez al frontend, que
  navega a `/scan?path=<ruta>` y el escaneo arranca automáticamente (acción
  explícita del usuario, no automática).
- Comandos Tauri: `install_context_menu`, `uninstall_context_menu`,
  `is_context_menu_installed`, `take_launch_path`.
