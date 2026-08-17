//! Generación de informes de análisis en HTML y CSV (FASE 8).
//!
//! Los informes se generan a partir de los resultados ya almacenados en
//! memoria: el módulo NUNCA vuelve a escanear ni consulta VirusTotal.
//! El HTML es autocontenido (CSS embebido, sin scripts ni recursos
//! externos) y todos los datos provenientes de usuario/archivos se escapan
//! para evitar inyección de HTML.

use serde_json::Value;

use crate::models::{
    AiAssessment, FolderFileEntry, FolderScanResult, ReportFormat, ScanResult, Severity,
    StaticAnalysis, ThreatLevel, VirusTotalResult,
};

fn severity_name(s: &Severity) -> String {
    match s {
        Severity::Info => "Info".into(),
        Severity::Low => "Low".into(),
        Severity::Medium => "Medium".into(),
        Severity::High => "High".into(),
        Severity::Critical => "Critical".into(),
    }
}

fn level_name(l: &ThreatLevel) -> String {
    match l {
        ThreatLevel::Clean => "Clean".into(),
        ThreatLevel::Low => "Low".into(),
        ThreatLevel::Medium => "Medium".into(),
        ThreatLevel::High => "High".into(),
        ThreatLevel::Critical => "Critical".into(),
    }
}

// ---------------------------------------------------------------------------
// Punto de entrada
// ---------------------------------------------------------------------------

/// Renderiza un resultado (archivo o carpeta) en el formato pedido.
pub fn render(value: &Value, format: ReportFormat) -> Result<String, String> {
    if value.get("folderPath").is_some() {
        let result: FolderScanResult = serde_json::from_value(value.clone())
            .map_err(|e| format!("Informe no disponible: {e}"))?;
        Ok(match format {
            ReportFormat::Html => render_folder_html(&result),
            ReportFormat::Csv => render_folder_csv(&result),
        })
    } else {
        let result: ScanResult = serde_json::from_value(value.clone())
            .map_err(|e| format!("Informe no disponible: {e}"))?;
        Ok(match format {
            ReportFormat::Html => render_file_html(&result),
            ReportFormat::Csv => render_file_csv(&result),
        })
    }
}

// ---------------------------------------------------------------------------
// HTML: archivo individual
// ---------------------------------------------------------------------------

pub fn render_file_html(r: &ScanResult) -> String {
    let mut body = String::new();

    // Cabecera
    body.push_str(&format!(
        "<header><div class=\"brand\">\u{1f6e1}\u{fe0f} VirusAnalyzer 2.0</div>\
         <div class=\"meta\">Report generated {}</div></header>\n",
        escape_html(&r.scanned_at)
    ));

    // Resumen
    body.push_str("<section class=\"card\"><h2>Summary</h2>\n");
    body.push_str(&format!("<h1>{}</h1>\n", escape_html(&r.file_name)));
    body.push_str(&format!("<p class=\"path\">{}</p>\n", escape_html(&r.path)));
    body.push_str("<div class=\"grid\">\n");
    body.push_str(&stat_cell("File size", &fmt_bytes(r.size)));
    body.push_str(&stat_cell(
        "Threat score",
        &format!("{}/100", r.threat_score),
    ));
    body.push_str(&stat_cell("Threat level", &level_badge(&r.threat_level)));
    body.push_str(&stat_cell("Scanned at", &escape_html(&r.scanned_at)));
    body.push_str("</div>\n</section>\n");

    // Evaluación
    if let Some(a) = &r.ai_assessment {
        body.push_str(&render_assessment_html(a));
    }

    // Hallazgos
    body.push_str("<section class=\"card\"><h2>Findings</h2>\n");
    if r.findings.is_empty() {
        body.push_str("<p class=\"none\">No heuristic findings.</p>\n");
    } else {
        body.push_str(
            "<table><thead><tr><th>Severity</th><th>Rule</th><th>Category</th>\
                       <th>Evidence</th><th>Points</th></tr></thead><tbody>\n",
        );
        for f in &r.findings {
            body.push_str(&format!(
                "<tr><td>{}</td><td><span class=\"mono\">{}</span></td><td>{}</td><td>{}</td>\
                 <td class=\"num\">{}</td></tr>\n",
                severity_badge(&f.severity),
                escape_html(&f.rule_name),
                escape_html(&f.category),
                escape_html(&f.evidence.join("; ")),
                f.points
            ));
        }
        body.push_str("</tbody></table>\n");
    }
    body.push_str("</section>\n");

    // Análisis estático
    if let Some(s) = &r.static_analysis {
        body.push_str(&render_static_html(s));
    }

    // Reputación
    if let Some(vt) = &r.reputation {
        body.push_str(&render_reputation_html(vt));
    }

    // Hashes
    body.push_str("<section class=\"card\"><h2>Hashes</h2>\n<table>\n");
    body.push_str(&hash_row("MD5", &r.hashes.md5));
    body.push_str(&hash_row("SHA-1", &r.hashes.sha1));
    body.push_str(&hash_row("SHA-256", &r.hashes.sha256));
    body.push_str("</table>\n</section>\n");

    // Línea temporal
    body.push_str("<section class=\"card\"><h2>Timeline</h2>\n<ol>\n");
    for e in &r.timeline {
        body.push_str(&format!(
            "<li><span class=\"mono\">{}</span> &mdash; {}</li>\n",
            escape_html(&e.time),
            escape_html(&e.label)
        ));
    }
    body.push_str("</ol>\n</section>\n");

    render_document(
        &format!("VirusAnalyzer report \u{2014} {}", r.file_name),
        &body,
    )
}

fn render_assessment_html(a: &AiAssessment) -> String {
    let mut out =
        String::from("<section class=\"card\"><h2>AI assessment</h2>\n<div class=\"grid\">\n");
    out.push_str(&stat_cell("Verdict", &verdict_badge(&a.verdict)));
    out.push_str(&stat_cell(
        "Confidence",
        &format!("{:.0}%", a.confidence * 100.0),
    ));
    out.push_str("</div>\n");
    out.push_str(&format!(
        "<p class=\"summary\">{}</p>\n",
        escape_html(&a.summary)
    ));
    for para in a.explanation.split("\n\n") {
        let para = para.trim();
        if !para.is_empty() {
            out.push_str(&format!("<p>{}</p>\n", escape_html(para)));
        }
    }
    if !a.indicators.is_empty() {
        out.push_str("<h3>Indicators</h3>\n<ul>\n");
        for i in &a.indicators {
            out.push_str(&format!("<li>{}</li>\n", escape_html(i)));
        }
        out.push_str("</ul>\n");
    }
    if !a.recommended_actions.is_empty() {
        out.push_str("<h3>Recommended actions</h3>\n<ul>\n");
        for x in &a.recommended_actions {
            out.push_str(&format!("<li>{}</li>\n", escape_html(x)));
        }
        out.push_str("</ul>\n");
    }
    out.push_str("</section>\n");
    out
}

fn render_static_html(s: &StaticAnalysis) -> String {
    let mut out =
        String::from("<section class=\"card\"><h2>Static analysis</h2>\n<div class=\"grid\">\n");
    out.push_str(&stat_cell(
        "File type",
        &format!(
            "{} ({}){}",
            escape_html(&s.file_type),
            escape_html(&s.file_type_extension),
            if s.type_mismatch {
                " \u{2014} mismatch"
            } else {
                ""
            }
        ),
    ));
    out.push_str(&stat_cell(
        "Entropy",
        &format!("{:.2} bits/byte", s.entropy),
    ));
    out.push_str("</div>\n");
    if let Some(pe) = &s.pe {
        out.push_str(&format!(
            "<p>PE {} &middot; {} &middot; {}{}</p>\n",
            if pe.is_dll { "DLL" } else { "executable" },
            escape_html(&pe.architecture),
            if pe.is_console { "console" } else { "GUI" },
            if pe.has_certificate {
                " &middot; signed"
            } else {
                ""
            }
        ));
        if !pe.imports.is_empty() {
            let imports = pe
                .imports
                .iter()
                .map(|d| format!("{} ({} functions)", d.name, d.functions.len()))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "<h3>Imports ({})</h3>\n<p class=\"wrap\">{}</p>\n",
                pe.imports.len(),
                escape_html(&imports)
            ));
        }
        if !pe.sections.is_empty() {
            out.push_str(&format!(
                "<h3>Sections ({})</h3>\n<p class=\"wrap\">{}</p>\n",
                pe.sections.len(),
                escape_html(
                    &pe.sections
                        .iter()
                        .map(|s| s.name.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            ));
        }
    }
    if !s.keywords.is_empty() {
        out.push_str("<h3>Keywords</h3>\n<p class=\"wrap\">");
        for k in &s.keywords {
            out.push_str(&format!("<span class=\"chip\">{}</span> ", escape_html(k)));
        }
        out.push_str("</p>\n");
    }
    out.push_str("</section>\n");
    out
}

fn render_reputation_html(vt: &VirusTotalResult) -> String {
    let mut out = String::from("<section class=\"card\"><h2>VirusTotal reputation</h2>\n");
    if vt.error.is_some() {
        out.push_str(&format!(
            "<p class=\"none\">Not available: {}</p>\n",
            escape_html(vt.error.as_deref().unwrap_or("unknown"))
        ));
        out.push_str("</section>\n");
        return out;
    }
    if !vt.available {
        out.push_str("<p class=\"none\">Hash not reported in VirusTotal.</p>\n</section>\n");
        return out;
    }
    out.push_str("<div class=\"grid\">\n");
    out.push_str(&stat_cell("Malicious", &vt.malicious.to_string()));
    out.push_str(&stat_cell("Suspicious", &vt.suspicious.to_string()));
    out.push_str(&stat_cell("Harmless", &vt.harmless.to_string()));
    out.push_str(&stat_cell("Undetected", &vt.undetected.to_string()));
    out.push_str(&stat_cell("Total engines", &vt.total.to_string()));
    out.push_str("</div>\n");
    if !vt.threat_names.is_empty() {
        out.push_str("<h3>Threat names</h3>\n<p class=\"wrap\">");
        for name in &vt.threat_names {
            out.push_str(&format!(
                "<span class=\"chip danger\">{}</span> ",
                escape_html(name)
            ));
        }
        out.push_str("</p>\n");
    }
    out.push_str(&format!(
        "<p class=\"path\"><a href=\"{}\">View on VirusTotal</a></p>\n",
        escape_html(&vt.permalink)
    ));
    out.push_str("</section>\n");
    out
}

// ---------------------------------------------------------------------------
// HTML: carpeta
// ---------------------------------------------------------------------------

pub fn render_folder_html(r: &FolderScanResult) -> String {
    let mut body = String::new();
    body.push_str(&format!(
        "<header><div class=\"brand\">\u{1f6e1}\u{fe0f} VirusAnalyzer 2.0</div>\
         <div class=\"meta\">Folder report generated {}</div></header>\n",
        escape_html(&r.scanned_at)
    ));
    body.push_str("<section class=\"card\"><h2>Folder summary</h2>\n");
    body.push_str(&format!("<h1>{}</h1>\n", escape_html(&r.folder_path)));
    body.push_str("<div class=\"grid\">\n");
    body.push_str(&stat_cell("Files found", &r.file_count.to_string()));
    body.push_str(&stat_cell("Scanned", &r.scanned_count.to_string()));
    body.push_str(&stat_cell("Skipped", &r.skipped_count.to_string()));
    body.push_str(&stat_cell("Errors", &r.error_count.to_string()));
    body.push_str(&stat_cell("Total size", &fmt_bytes(r.total_bytes)));
    body.push_str(&stat_cell("Duration", &format!("{} ms", r.duration_ms)));
    body.push_str("</div>\n</section>\n");

    body.push_str(
        "<section class=\"card\"><h2>Files</h2>\n<table><thead><tr>\
                   <th>File</th><th>Size</th><th>MD5</th><th>SHA-1</th><th>SHA-256</th>\
                   <th>Status</th></tr></thead><tbody>\n",
    );
    for f in &r.files {
        body.push_str(&folder_file_row(f));
    }
    if r.files.is_empty() {
        body.push_str("<tr><td colspan=\"6\" class=\"none\">No files found.</td></tr>\n");
    }
    body.push_str("</tbody></table>\n</section>\n");

    render_document(
        &format!("VirusAnalyzer folder report \u{2014} {}", r.folder_path),
        &body,
    )
}

fn folder_file_row(f: &FolderFileEntry) -> String {
    if f.error.is_some() {
        return format!(
            "<tr><td class=\"mono\">{}</td><td>{}</td><td colspan=\"3\"></td>\
             <td class=\"err\">error</td></tr>\n",
            escape_html(&f.relative_path),
            fmt_bytes(f.size),
        );
    }
    format!(
        "<tr><td class=\"mono\">{}</td><td>{}</td><td class=\"mono\">{}</td>\
         <td class=\"mono\">{}</td><td class=\"mono\">{}</td><td>ok</td></tr>\n",
        escape_html(&f.relative_path),
        fmt_bytes(f.size),
        hash_or_dash(&f.hashes.md5),
        hash_or_dash(&f.hashes.sha1),
        hash_or_dash(&f.hashes.sha256),
    )
}

// ---------------------------------------------------------------------------
// CSV
// ---------------------------------------------------------------------------

pub fn render_file_csv(r: &ScanResult) -> String {
    let findings = r
        .findings
        .iter()
        .map(|f| {
            format!(
                "[{}] {} ({})",
                severity_name(&f.severity),
                f.rule_name,
                f.category
            )
        })
        .collect::<Vec<_>>()
        .join(" | ");
    let evidence = r
        .findings
        .iter()
        .flat_map(|f| f.evidence.clone())
        .collect::<Vec<_>>()
        .join(" | ");
    let verdict = r
        .ai_assessment
        .as_ref()
        .map(|a| a.verdict.clone())
        .unwrap_or_default();
    let confidence = r
        .ai_assessment
        .as_ref()
        .map(|a| format!("{:.0}%", a.confidence * 100.0))
        .unwrap_or_default();
    let summary = r
        .ai_assessment
        .as_ref()
        .map(|a| a.summary.clone())
        .unwrap_or_default();
    let vt = r.reputation.as_ref();
    let vt_malicious = vt.map(|v| v.malicious).unwrap_or(0);
    let vt_total = vt.map(|v| v.total).unwrap_or(0);
    let vt_names = vt.map(|v| v.threat_names.join("; ")).unwrap_or_default();

    let headers = [
        "id",
        "fileName",
        "path",
        "sizeBytes",
        "scannedAt",
        "threatScore",
        "threatLevel",
        "verdict",
        "confidence",
        "summary",
        "md5",
        "sha1",
        "sha256",
        "findingsCount",
        "findings",
        "evidence",
        "vtMalicious",
        "vtTotal",
        "vtThreatNames",
    ];
    let row = [
        r.id.clone(),
        r.file_name.clone(),
        r.path.clone(),
        r.size.to_string(),
        r.scanned_at.clone(),
        r.threat_score.to_string(),
        level_name(&r.threat_level),
        verdict,
        confidence,
        summary,
        r.hashes.md5.clone().unwrap_or_default(),
        r.hashes.sha1.clone().unwrap_or_default(),
        r.hashes.sha256.clone().unwrap_or_default(),
        r.findings.len().to_string(),
        findings,
        evidence,
        vt_malicious.to_string(),
        vt_total.to_string(),
        vt_names,
    ];
    let mut out = String::new();
    out.push_str(&csv_row(&headers));
    out.push_str(&csv_row(&row));
    out
}

pub fn render_folder_csv(r: &FolderScanResult) -> String {
    let mut out = String::new();

    out.push_str(&csv_row(&["SECTION", "folder summary"]));
    out.push_str(&csv_row(&[
        "folderPath",
        "scannedAt",
        "fileCount",
        "scannedCount",
        "skippedCount",
        "errorCount",
        "totalBytes",
        "durationMs",
    ]));
    out.push_str(&csv_row(&[
        r.folder_path.clone(),
        r.scanned_at.clone(),
        r.file_count.to_string(),
        r.scanned_count.to_string(),
        r.skipped_count.to_string(),
        r.error_count.to_string(),
        r.total_bytes.to_string(),
        r.duration_ms.to_string(),
    ]));
    out.push('\n');

    out.push_str(&csv_row(&["SECTION", "files"]));
    out.push_str(&csv_row(&[
        "relativePath",
        "sizeBytes",
        "md5",
        "sha1",
        "sha256",
        "error",
    ]));
    for f in &r.files {
        out.push_str(&csv_row(&[
            f.relative_path.clone(),
            f.size.to_string(),
            f.hashes.md5.clone().unwrap_or_default(),
            f.hashes.sha1.clone().unwrap_or_default(),
            f.hashes.sha256.clone().unwrap_or_default(),
            f.error.clone().unwrap_or_default(),
        ]));
    }
    out
}

// ---------------------------------------------------------------------------
// Helpers de renderizado
// ---------------------------------------------------------------------------

fn render_document(title: &str, body: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{title}</title>\n<style>\n{CSS}\n</style>\n</head>\n<body>\n{body}\
         <footer>Generated by VirusAnalyzer 2.0 &mdash; evidence-based analysis; scores are \
         guidance, not proof of intent. This report contains no executable content.</footer>\n\
         </body>\n</html>\n",
        title = escape_html(title),
        body = body,
    )
}

const CSS: &str = r#"
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:-apple-system,"Segoe UI",Roboto,Arial,sans-serif;color:#1b2430;background:#f2f4f8;padding:24px;line-height:1.55}
header{display:flex;justify-content:space-between;align-items:center;margin-bottom:20px;flex-wrap:wrap;gap:8px}
.brand{font-size:18px;font-weight:700}
.meta{font-size:12px;color:#667}
main{max-width:960px;margin:0 auto}
footer{margin-top:28px;font-size:11px;color:#889;text-align:center}
.card{background:#fff;border:1px solid #e3e7ee;border-radius:10px;padding:18px 20px;margin-bottom:16px}
h1{font-size:20px;margin:4px 0 2px;word-break:break-all}
h2{font-size:14px;text-transform:uppercase;letter-spacing:.05em;color:#556;margin-bottom:12px}
h3{font-size:12px;margin:14px 0 6px;color:#667}
.path{font-size:11px;color:#889;word-break:break-all;margin-bottom:10px}
.summary{font-size:14px;margin:8px 0}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(150px,1fr));gap:10px;margin:12px 0}
.stat{background:#f7f9fc;border:1px solid #edf0f5;border-radius:8px;padding:10px 12px}
.stat .label{font-size:10px;text-transform:uppercase;color:#889;letter-spacing:.05em}
.stat .value{font-size:16px;font-weight:600;margin-top:2px}
table{width:100%;border-collapse:collapse;font-size:12px}
th{text-align:left;font-size:10px;text-transform:uppercase;color:#889;border-bottom:1px solid #e3e7ee;padding:6px 8px}
td{padding:6px 8px;border-bottom:1px solid #f0f2f6;vertical-align:top;word-break:break-all}
tr:last-child td{border-bottom:none}
td.num{text-align:right;font-variant-numeric:tabular-nums}
.mono{font-family:ui-monospace,Consolas,monospace;font-size:11px}
.wrap{word-break:break-all}
.chip{display:inline-block;background:#eef2f8;border:1px solid #dde3ec;border-radius:99px;padding:2px 8px;font-size:11px;margin:2px}
.chip.danger{background:#fdecec;border-color:#f3c1c1;color:#a02b2b}
.badge{display:inline-block;border-radius:99px;padding:2px 10px;font-size:11px;font-weight:600}
.lv-clean{background:#e6f6ec;color:#177a3a}
.lv-low{background:#e9f4fa;color:#16658f}
.lv-moderate{background:#fff3e0;color:#a05c00}
.lv-high{background:#fdecea;color:#b03030}
.lv-critical{background:#c33;color:#fff}
.sev-info{color:#16658f}.sev-low{color:#177a3a}.sev-medium{color:#a05c00}.sev-high{color:#b03030}.sev-critical{color:#c33;font-weight:700}
.v-clean{background:#e6f6ec;color:#177a3a}.v-likely-clean{background:#e9f4fa;color:#16658f}
.v-suspicious{background:#fff3e0;color:#a05c00}.v-malicious{background:#c33;color:#fff}
.none{color:#889;font-size:12px}
.err{color:#c33;font-weight:600}
ol{padding-left:20px;font-size:12px;color:#334}
a{color:#16658f}
"#;

fn stat_cell(label: &str, value: &str) -> String {
    format!(
        "<div class=\"stat\"><div class=\"label\">{}</div><div class=\"value\">{}</div></div>\n",
        escape_html(label),
        value
    )
}

fn level_badge(level: &ThreatLevel) -> String {
    let name = level_name(level);
    format!(
        "<span class=\"badge lv-{}\">{}</span>",
        name.to_lowercase(),
        escape_html(&name)
    )
}

fn severity_badge(sev: &Severity) -> String {
    let name = severity_name(sev);
    format!(
        "<span class=\"sev-{}\">{}</span>",
        name.to_lowercase(),
        escape_html(&name)
    )
}

fn verdict_badge(verdict: &str) -> String {
    format!(
        "<span class=\"badge v-{}\">{}</span>",
        verdict.to_lowercase().replace(' ', "-"),
        escape_html(verdict)
    )
}

fn hash_row(label: &str, hash: &Option<String>) -> String {
    format!(
        "<tr><th>{}</th><td class=\"mono\">{}</td></tr>\n",
        escape_html(label),
        hash_or_dash(hash)
    )
}

fn hash_or_dash(hash: &Option<String>) -> String {
    match hash {
        Some(h) if !h.is_empty() => escape_html(h),
        _ => "\u{2014}".to_string(),
    }
}

fn fmt_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if value >= 100.0 {
        format!("{:.0} {}", value, UNITS[unit])
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn csv_row<S: AsRef<str>>(fields: &[S]) -> String {
    let mut out = String::new();
    for (i, f) in fields.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&csv_field(f.as_ref()));
    }
    out.push('\n');
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FileHashes, Language, TimelineEntry};

    fn sample_result() -> ScanResult {
        ScanResult {
            id: "S-2026-000001".into(),
            file_name: "sample.exe".into(),
            path: r"C:\Temp\sample.exe".into(),
            size: 123456,
            hashes: FileHashes {
                md5: Some("deadbeef".into()),
                sha1: None,
                sha256: Some("cafebabe".into()),
            },
            threat_score: 55,
            threat_level: ThreatLevel::High,
            findings: vec![],
            static_analysis: None,
            reputation: None,
            ai_assessment: None,
            language: Language::En,
            scanned_at: "2026-08-13T10:00:00Z".into(),
            timeline: vec![TimelineEntry {
                time: "2026-08-13T10:00:00Z".into(),
                label: "Scan started".into(),
            }],
        }
    }

    #[test]
    fn file_html_escapes_dangerous_names() {
        let mut r = sample_result();
        r.file_name = "<script>alert(1)</script> & \"file\".exe".into();
        let html = render_file_html(&r);
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn file_html_contains_core_sections() {
        let html = render_file_html(&sample_result());
        assert!(html.contains(">Summary<"));
        assert!(html.contains(">Hashes<"));
        assert!(html.contains(">Timeline<"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn file_csv_escapes_commas_and_quotes() {
        let mut r = sample_result();
        r.file_name = "a,b\"c".into();
        let csv = render_file_csv(&r);
        assert!(csv.contains("\"a,b\"\"c\""));
    }

    #[test]
    fn folder_csv_has_two_sections() {
        let r = FolderScanResult {
            id: "S-2026-000002".into(),
            folder_path: r"C:\Temp".into(),
            file_count: 1,
            scanned_count: 1,
            skipped_count: 0,
            error_count: 0,
            total_bytes: 10,
            scanned_at: "2026-08-13T10:00:00Z".into(),
            duration_ms: 5,
            files: vec![FolderFileEntry {
                relative_path: "a.bin".into(),
                size: 10,
                hashes: FileHashes {
                    md5: Some("aa".into()),
                    sha1: None,
                    sha256: None,
                },
                error: None,
            }],
        };
        let csv = render_folder_csv(&r);
        assert!(csv.contains("folder summary"));
        assert!(csv.contains("SECTION,files"));
        assert!(csv.contains("a.bin"));
    }

    #[test]
    fn render_dispatches_by_folder_flag() {
        let r = sample_result();
        let value = serde_json::to_value(&r).unwrap();
        let html = render(&value, ReportFormat::Html).unwrap();
        assert!(html.contains("Summary"));

        let f = FolderScanResult {
            id: "S-2026-000003".into(),
            folder_path: r"C:\Temp".into(),
            file_count: 0,
            scanned_count: 0,
            skipped_count: 0,
            error_count: 0,
            total_bytes: 0,
            scanned_at: "2026-08-13T10:00:00Z".into(),
            duration_ms: 0,
            files: vec![],
        };
        let value = serde_json::to_value(&f).unwrap();
        let html = render(&value, ReportFormat::Html).unwrap();
        assert!(html.contains("Folder summary"));
    }
}
