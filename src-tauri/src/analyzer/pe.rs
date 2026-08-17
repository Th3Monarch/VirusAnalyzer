//! Parser de archivos PE (Portable Executable) de Windows.
//!
//! Subconjunto enfocado en seguridad: cabeceras DOS/NT, secciones, imports,
//! exports, certificado Authenticode y subsistema. No se ejecuta nada.

use chrono::{DateTime, Utc};

use crate::analyzer;
use crate::models::{PeImportDll, PeInfo, PeSection};

const DOS_HEADER_SIZE: usize = 64;
const COFF_OFFSET_IN_NT: usize = 4; // firma NT (4 bytes) + COFF header
const OPTIONAL_OFFSET_IN_NT: usize = 24; // 4 (firma) + 20 (COFF)

const PE32_MAGIC: u16 = 0x010b;
const PE32_PLUS_MAGIC: u16 = 0x020b;

const SECTION_HEADER_SIZE: usize = 40;
const IMPORT_DESCRIPTOR_SIZE: usize = 20;

// Límites de visualización (el payload JSON debe ser manejable).
const MAX_IMPORT_DLLS: usize = 200;
const MAX_FUNCTIONS_PER_DLL: usize = 100;
const MAX_EXPORTS: usize = 300;

/// Comprueba de forma barata (primeros bytes) si un archivo parece un PE.
pub fn looks_like_pe(head: &[u8]) -> bool {
    if head.len() < DOS_HEADER_SIZE || &head[0..2] != b"MZ" {
        return false;
    }
    let e_lfanew = le_u32(head, 0x3c) as usize;
    e_lfanew + 4 <= head.len() && &head[e_lfanew..e_lfanew + 4] == b"PE\0\0"
}

/// Analiza un buffer completo de un archivo PE.
pub fn parse(bytes: &[u8]) -> Option<PeInfo> {
    if bytes.len() < DOS_HEADER_SIZE || &bytes[0..2] != b"MZ" {
        return None;
    }
    let e_lfanew = le_u32(bytes, 0x3c) as usize;
    if e_lfanew + 4 > bytes.len() || &bytes[e_lfanew..e_lfanew + 4] != b"PE\0\0" {
        return None;
    }

    let coff = e_lfanew + COFF_OFFSET_IN_NT;
    if coff + 20 > bytes.len() {
        return None;
    }

    let machine = le_u16(bytes, coff);
    let number_of_sections = le_u16(bytes, coff + 2) as usize;
    let timestamp = le_u32(bytes, coff + 8) as u64;
    let characteristics = le_u16(bytes, coff + 18);
    let size_of_optional = le_u16(bytes, coff + 16) as usize;

    let optional = e_lfanew + OPTIONAL_OFFSET_IN_NT;
    if optional + 2 > bytes.len() {
        return None;
    }
    let magic = le_u16(bytes, optional);
    let is_plus = magic == PE32_PLUS_MAGIC;
    if magic != PE32_MAGIC && !is_plus {
        return None;
    }

    let entry_point = le_u32(bytes, optional + 16) as u64;
    let image_base = if is_plus {
        le_u64(bytes, optional + 24)
    } else {
        le_u32(bytes, optional + 28) as u64
    };
    let subsystem_raw = le_u16(bytes, optional + 68);
    let dll_characteristics = le_u16(bytes, optional + 70) as u32;
    let number_of_rva_and_sizes = if is_plus {
        le_u32(bytes, optional + 108)
    } else {
        le_u32(bytes, optional + 92)
    } as usize;
    let data_directory = if is_plus {
        optional + 112
    } else {
        optional + 96
    };

    let sections_start = optional + size_of_optional;

    // Secciones.
    let mut sections: Vec<PeSection> = Vec::new();
    // (nombre, VA, VirtualSize, SizeOfRawData, PointerToRawData) para RVA→offset.
    let mut secs: Vec<(String, u32, u32, u32, usize)> = Vec::new();

    for i in 0..number_of_sections {
        let base = sections_start + i * SECTION_HEADER_SIZE;
        if base + SECTION_HEADER_SIZE > bytes.len() {
            break;
        }
        let name = read_name(&bytes[base..base + 8]);
        let virtual_size = le_u32(bytes, base + 8) as u64;
        let virtual_address = le_u32(bytes, base + 12) as u64;
        let raw_size = le_u32(bytes, base + 16) as u64;
        let raw_ptr = le_u32(bytes, base + 20) as usize;
        let flags = section_flags(le_u32(bytes, base + 36));
        let entropy = section_entropy(bytes, raw_ptr, raw_size);

        sections.push(PeSection {
            name,
            virtual_size,
            virtual_address,
            raw_size,
            entropy,
            flags,
        });
        secs.push((
            sections.last().unwrap().name.clone(),
            virtual_address as u32,
            virtual_size as u32,
            raw_size as u32,
            raw_ptr,
        ));
    }

    // Directorio de seguridad (certificado Authenticode).
    let (has_certificate, certificate_size) =
        match read_directory(bytes, data_directory, number_of_rva_and_sizes, 4) {
            Some((_rva, size)) => (size > 0, size),
            None => (false, 0),
        };

    // Imports.
    let mut imports: Vec<PeImportDll> = Vec::new();
    if let Some((rva, _size)) = read_directory(bytes, data_directory, number_of_rva_and_sizes, 1) {
        imports = parse_imports(bytes, rva, is_plus, &secs);
    }
    let import_count = imports.iter().map(|d| d.functions.len() as u32).sum();

    // Exports.
    let mut exports: Vec<String> = Vec::new();
    if let Some((rva, _size)) = read_directory(bytes, data_directory, number_of_rva_and_sizes, 0) {
        exports = parse_exports(bytes, rva, &secs);
    }
    let export_count = exports.len() as u32;

    let is_dll = characteristics & 0x2000 != 0;
    let is_executable = characteristics & 0x0002 != 0;
    let (is_console, is_gui) = match subsystem_raw {
        2 => (false, true),
        3 => (true, false),
        _ => (false, false),
    };

    Some(PeInfo {
        machine: machine_string(machine),
        architecture: arch_string(machine),
        is_dll,
        is_executable,
        is_console,
        is_gui,
        image_base,
        entry_point,
        timestamp,
        timestamp_iso: timestamp_iso(timestamp),
        subsystem: subsystem_string(subsystem_raw),
        dll_characteristics,
        has_certificate,
        certificate_size,
        sections,
        imports,
        import_count,
        exports,
        export_count,
    })
}

fn read_directory(
    bytes: &[u8],
    data_directory: usize,
    count: usize,
    index: usize,
) -> Option<(u32, u32)> {
    if index >= count {
        return None;
    }
    // Cada entrada del data directory son 8 bytes: {RVA u32, Size u32}.
    let base = data_directory + index * 8;
    if base + 8 > bytes.len() {
        return None;
    }
    let rva = le_u32(bytes, base);
    let size = le_u32(bytes, base + 4);
    if rva == 0 {
        return None;
    }
    Some((rva, size))
}

/// Convierte un RVA en un offset de archivo usando la tabla de secciones.
fn rva_to_offset(bytes: &[u8], rva: u32, secs: &[(String, u32, u32, u32, usize)]) -> Option<usize> {
    for (_name, va, vsize, rawsize, ptr) in secs {
        let span = (*vsize).max(*rawsize);
        if *va <= rva && rva < va + span {
            let off = *ptr + (rva - va) as usize;
            if off < bytes.len() {
                return Some(off);
            }
        }
    }
    None
}

fn parse_imports(
    bytes: &[u8],
    dir_rva: u32,
    is_plus: bool,
    secs: &[(String, u32, u32, u32, usize)],
) -> Vec<PeImportDll> {
    let mut out: Vec<PeImportDll> = Vec::new();
    let Some(mut off) = rva_to_offset(bytes, dir_rva, secs) else {
        return out;
    };

    for _ in 0..4096 {
        if off + IMPORT_DESCRIPTOR_SIZE > bytes.len() {
            break;
        }
        let original_first_thunk = le_u32(bytes, off);
        let name_rva = le_u32(bytes, off + 12);
        let first_thunk = le_u32(bytes, off + 16);
        if original_first_thunk == 0 && name_rva == 0 && first_thunk == 0 {
            break;
        }
        let Some(name_off) = rva_to_offset(bytes, name_rva, secs) else {
            break;
        };
        let name = read_cstring(bytes, name_off).unwrap_or_default();
        if name.is_empty() {
            break;
        }
        let thunk_rva = if original_first_thunk != 0 {
            original_first_thunk
        } else {
            first_thunk
        };
        let functions = parse_thunks(bytes, thunk_rva, is_plus, secs);
        out.push(PeImportDll { name, functions });
        if out.len() >= MAX_IMPORT_DLLS {
            break;
        }
        off += IMPORT_DESCRIPTOR_SIZE;
    }
    out
}

fn parse_thunks(
    bytes: &[u8],
    thunk_rva: u32,
    is_plus: bool,
    secs: &[(String, u32, u32, u32, usize)],
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let size = if is_plus { 8 } else { 4 };
    let high_bit = if is_plus {
        0x8000_0000_0000_0000u64
    } else {
        0x8000_0000u64
    };
    let Some(mut off) = rva_to_offset(bytes, thunk_rva, secs) else {
        return out;
    };

    for _ in 0..8192 {
        if off + size > bytes.len() {
            break;
        }
        let value = if is_plus {
            le_u64(bytes, off)
        } else {
            le_u32(bytes, off) as u64
        };
        if value == 0 {
            break;
        }
        if value & high_bit != 0 {
            out.push(format!("Ordinal {}", value & 0xffff));
        } else if let Some(n_off) = rva_to_offset(bytes, value as u32, secs) {
            // IMAGE_IMPORT_BY_NAME: Hint (2) + Name.
            let name = read_cstring(bytes, n_off + 2).unwrap_or_default();
            if !name.is_empty() {
                out.push(name);
            }
        } else {
            out.push(format!("0x{value:x}"));
        }
        if out.len() >= MAX_FUNCTIONS_PER_DLL {
            break;
        }
        off += size;
    }
    out
}

fn parse_exports(
    bytes: &[u8],
    dir_rva: u32,
    secs: &[(String, u32, u32, u32, usize)],
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let Some(off) = rva_to_offset(bytes, dir_rva, secs) else {
        return out;
    };
    if off + 40 > bytes.len() {
        return out;
    }
    let number_of_names = le_u32(bytes, off + 24);
    let names_rva = le_u32(bytes, off + 32);
    let Some(mut names_off) = rva_to_offset(bytes, names_rva, secs) else {
        return out;
    };

    for _ in 0..number_of_names.min(MAX_EXPORTS as u32) {
        if names_off + 4 > bytes.len() {
            break;
        }
        let n_rva = le_u32(bytes, names_off);
        if let Some(n_off) = rva_to_offset(bytes, n_rva, secs) {
            if let Some(s) = read_cstring(bytes, n_off) {
                out.push(s);
            }
        }
        names_off += 4;
    }
    out
}

/// Entropía de la porción de la sección presente en disco.
fn section_entropy(bytes: &[u8], raw_ptr: usize, raw_size: u64) -> f64 {
    let end = (raw_ptr as u64 + raw_size).min(bytes.len() as u64) as usize;
    if raw_ptr >= end {
        return 0.0;
    }
    analyzer::entropy(&bytes[raw_ptr..end])
}

fn section_flags(ch: u32) -> Vec<String> {
    let mut f = Vec::new();
    if ch & 0x0000_0020 != 0 {
        f.push("CODE".into());
    }
    if ch & 0x0000_0040 != 0 {
        f.push("INIT_DATA".into());
    }
    if ch & 0x0000_0080 != 0 {
        f.push("UNINIT_DATA".into());
    }
    if ch & 0x2000_0000 != 0 {
        f.push("EXEC".into());
    }
    if ch & 0x4000_0000 != 0 {
        f.push("READ".into());
    }
    if ch & 0x8000_0000 != 0 {
        f.push("WRITE".into());
    }
    if f.is_empty() {
        f.push("NONE".into());
    }
    f
}

fn arch_string(machine: u16) -> String {
    match machine {
        0x014c => "x86".into(),
        0x8664 => "x64".into(),
        0x01c0 | 0x01c4 => "arm".into(),
        0xaa64 => "arm64".into(),
        0x0200 => "ia64".into(),
        _ => "unknown".into(),
    }
}

fn machine_string(machine: u16) -> String {
    match machine {
        0x014c => "IMAGE_FILE_MACHINE_I386".into(),
        0x8664 => "IMAGE_FILE_MACHINE_AMD64".into(),
        0x01c0 => "IMAGE_FILE_MACHINE_ARM".into(),
        0x01c4 => "IMAGE_FILE_MACHINE_ARMNT".into(),
        0xaa64 => "IMAGE_FILE_MACHINE_ARM64".into(),
        0x0200 => "IMAGE_FILE_MACHINE_IA64".into(),
        _ => format!("0x{machine:04x}"),
    }
}

fn subsystem_string(v: u16) -> String {
    match v {
        1 => "Native".into(),
        2 => "Windows GUI".into(),
        3 => "Windows Console (CUI)".into(),
        7 => "POSIX CUI".into(),
        9 => "Windows CE GUI".into(),
        10 => "EFI Application".into(),
        _ => format!("Unknown ({v})"),
    }
}

fn timestamp_iso(secs: u64) -> String {
    DateTime::<Utc>::from_timestamp(secs as i64, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}

fn read_name(raw: &[u8]) -> String {
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..end]).into_owned()
}

fn read_cstring(bytes: &[u8], offset: usize) -> Option<String> {
    if offset >= bytes.len() {
        return None;
    }
    let end = bytes[offset..]
        .iter()
        .position(|&b| b == 0)
        .map(|p| offset + p)
        .unwrap_or(bytes.len());
    let len = (end - offset).min(256);
    Some(String::from_utf8_lossy(&bytes[offset..offset + len]).into_owned())
}

fn le_u16(b: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([*b.get(off).unwrap_or(&0), *b.get(off + 1).unwrap_or(&0)])
}

fn le_u32(b: &[u8], off: usize) -> u32 {
    let mut a = [0u8; 4];
    for (i, byte) in a.iter_mut().enumerate() {
        *byte = *b.get(off + i).unwrap_or(&0);
    }
    u32::from_le_bytes(a)
}

fn le_u64(b: &[u8], off: usize) -> u64 {
    let mut a = [0u8; 8];
    for (i, byte) in a.iter_mut().enumerate() {
        *byte = *b.get(off + i).unwrap_or(&0);
    }
    u64::from_le_bytes(a)
}
