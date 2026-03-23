// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RecoveryConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ClassificationBasis {
    MagicBytes,
    Structure,
    Heuristic,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ContentClassification {
    pub(crate) kind: String,
    pub(crate) confidence: RecoveryConfidence,
    pub(crate) basis: ClassificationBasis,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) subtype: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct NamingDecision {
    pub(crate) assigned_name: String,
    pub(crate) classification: ContentClassification,
}

#[derive(Debug, Clone, Copy)]
struct SignatureRule {
    kind: &'static str,
    ext: &'static str,
    signature: &'static [u8],
    basis: ClassificationBasis,
    confidence: RecoveryConfidence,
}

const MAGIC_RULES: &[SignatureRule] = &[
    SignatureRule {
        kind: "png",
        ext: "png",
        signature: b"\x89PNG\r\n\x1a\n",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "jpeg",
        ext: "jpg",
        signature: b"\xFF\xD8\xFF",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "gif",
        ext: "gif",
        signature: b"GIF87a",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "gif",
        ext: "gif",
        signature: b"GIF89a",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "pdf",
        ext: "pdf",
        signature: b"%PDF-",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "gzip",
        ext: "gz",
        signature: b"\x1f\x8b\x08",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "bzip2",
        ext: "bz2",
        signature: b"BZh",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "xz",
        ext: "xz",
        signature: b"\xFD7zXZ\0",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "7z",
        ext: "7z",
        signature: b"7z\xBC\xAF\x27\x1C",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "rar",
        ext: "rar",
        signature: b"Rar!\x1A\x07\x00",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "rar",
        ext: "rar",
        signature: b"Rar!\x1A\x07\x01\x00",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "cab",
        ext: "cab",
        signature: b"MSCF",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "sqlite",
        ext: "sqlite",
        signature: b"SQLite format 3\0",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "elf",
        ext: "elf",
        signature: b"\x7FELF",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "wasm",
        ext: "wasm",
        signature: b"\0asm",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "flac",
        ext: "flac",
        signature: b"fLaC",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "ogg",
        ext: "ogg",
        signature: b"OggS",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "bmp",
        ext: "bmp",
        signature: b"BM",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "rtf",
        ext: "rtf",
        signature: b"{\\rtf",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "postscript",
        ext: "ps",
        signature: b"%!PS-",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "rpm",
        ext: "rpm",
        signature: b"\xED\xAB\xEE\xDB",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "ar",
        ext: "ar",
        signature: b"!<arch>\n",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "qcow2",
        ext: "qcow2",
        signature: b"QFI\xFB",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::High,
    },
    SignatureRule {
        kind: "vmdk",
        ext: "vmdk",
        signature: b"KDMV",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::Medium,
    },
    SignatureRule {
        kind: "ole_cf",
        ext: "ole",
        signature: b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1",
        basis: ClassificationBasis::MagicBytes,
        confidence: RecoveryConfidence::Medium,
    },
];

pub(crate) fn classify_and_name(bytes: &[u8], id: usize) -> NamingDecision {
    let id_text = format!("{id:06}");
    let classification = classify_content(bytes);

    let assigned_name = match classification.confidence {
        RecoveryConfidence::High => {
            format!("file_{id_text}.{}", preferred_extension(&classification))
        }
        RecoveryConfidence::Medium => {
            format!(
                "file_{id_text}.probable-{}.bin",
                probable_label(&classification)
            )
        }
        RecoveryConfidence::Low => format!("file_{id_text}.bin"),
    };

    NamingDecision {
        assigned_name,
        classification,
    }
}

pub(crate) fn classify_content(bytes: &[u8]) -> ContentClassification {
    if let Some(structural) = classify_with_structure(bytes) {
        return structural;
    }

    if let Some(rule) = MAGIC_RULES
        .iter()
        .find(|rule| bytes.starts_with(rule.signature))
    {
        return ContentClassification {
            kind: rule.kind.to_string(),
            confidence: rule.confidence,
            basis: rule.basis,
            subtype: None,
        };
    }

    if looks_like_xml(bytes) {
        return ContentClassification {
            kind: "xml".to_string(),
            confidence: RecoveryConfidence::Medium,
            basis: ClassificationBasis::Heuristic,
            subtype: None,
        };
    }

    if looks_like_html(bytes) {
        return ContentClassification {
            kind: "html".to_string(),
            confidence: RecoveryConfidence::Medium,
            basis: ClassificationBasis::Heuristic,
            subtype: None,
        };
    }

    if serde_json::from_slice::<serde_json::Value>(bytes).is_ok() {
        return ContentClassification {
            kind: "json".to_string(),
            confidence: RecoveryConfidence::High,
            basis: ClassificationBasis::Structure,
            subtype: None,
        };
    }

    if looks_like_text(bytes) {
        return ContentClassification {
            kind: "text".to_string(),
            confidence: RecoveryConfidence::Medium,
            basis: ClassificationBasis::Heuristic,
            subtype: None,
        };
    }

    ContentClassification {
        kind: "bin".to_string(),
        confidence: RecoveryConfidence::Low,
        basis: ClassificationBasis::Heuristic,
        subtype: None,
    }
}

fn classify_with_structure(bytes: &[u8]) -> Option<ContentClassification> {
    if bytes.starts_with(b"RIFF") && bytes.len() >= 12 {
        let riff_kind = &bytes[8..12];
        if riff_kind == b"WAVE" {
            return Some(high("wav", ClassificationBasis::Structure, None));
        }
        if riff_kind == b"AVI " {
            return Some(high("avi", ClassificationBasis::Structure, None));
        }
        if riff_kind == b"WEBP" {
            return Some(high("webp", ClassificationBasis::Structure, None));
        }
    }

    if bytes.starts_with(&[0xFF, 0xF1]) || bytes.starts_with(&[0xFF, 0xF9]) {
        return Some(high("aac", ClassificationBasis::MagicBytes, None));
    }

    if bytes.starts_with(b"\x00\x00\x01\xBA") || bytes.starts_with(b"\x00\x00\x01\xB3") {
        return Some(medium("mpeg_ps", ClassificationBasis::MagicBytes, None));
    }

    if bytes.len() > 41000 && bytes[0x8001..0x8006] == *b"CD001" {
        return Some(high("iso", ClassificationBasis::Structure, None));
    }

    if bytes.len() > 262 && bytes[257..262] == *b"ustar" {
        return Some(high("tar", ClassificationBasis::Structure, None));
    }

    if is_tiff(bytes) {
        return Some(high("tiff", ClassificationBasis::MagicBytes, None));
    }

    if is_ico(bytes) {
        return Some(high("ico", ClassificationBasis::Structure, None));
    }

    if is_mp3(bytes) {
        return Some(medium("mp3", ClassificationBasis::Structure, None));
    }

    if is_mkv_or_webm(bytes) {
        let kind = if bytes.windows(4).any(|w| w == b"webm") {
            "webm"
        } else {
            "mkv"
        };
        return Some(medium(kind, ClassificationBasis::Structure, None));
    }

    if is_pe(bytes) {
        return Some(high("pe", ClassificationBasis::Structure, None));
    }

    if is_macho(bytes) {
        return Some(high("mach_o", ClassificationBasis::MagicBytes, None));
    }

    if bytes.starts_with(b"PK\x03\x04") {
        return Some(classify_zip_family(bytes));
    }

    if bytes.starts_with(b"!<arch>\n") && bytes.windows(11).any(|w| w == b"debian-bina") {
        return Some(high("deb", ClassificationBasis::Structure, Some("ar")));
    }

    if bytes.windows(9).any(|w| w == b"MANIFEST-") {
        return Some(medium("leveldb", ClassificationBasis::Heuristic, None));
    }

    None
}

fn classify_zip_family(bytes: &[u8]) -> ContentClassification {
    let has_content_types = contains_ascii(bytes, "[Content_Types].xml");
    let has_word = contains_ascii(bytes, "word/");
    let has_xl = contains_ascii(bytes, "xl/");
    let has_ppt = contains_ascii(bytes, "ppt/");

    if has_content_types && has_word {
        return high("docx", ClassificationBasis::Structure, Some("zip"));
    }
    if has_content_types && has_xl {
        return high("xlsx", ClassificationBasis::Structure, Some("zip"));
    }
    if has_content_types && has_ppt {
        return high("pptx", ClassificationBasis::Structure, Some("zip"));
    }

    if contains_ascii(bytes, "application/vnd.oasis.opendocument.text") {
        return high("odt", ClassificationBasis::Structure, Some("zip"));
    }
    if contains_ascii(bytes, "application/vnd.oasis.opendocument.spreadsheet") {
        return high("ods", ClassificationBasis::Structure, Some("zip"));
    }
    if contains_ascii(bytes, "application/vnd.oasis.opendocument.presentation") {
        return high("odp", ClassificationBasis::Structure, Some("zip"));
    }

    if contains_ascii(bytes, "application/epub+zip") {
        return high("epub", ClassificationBasis::Structure, Some("zip"));
    }

    if contains_ascii(bytes, "META-INF/MANIFEST.MF") {
        return high("jar", ClassificationBasis::Structure, Some("zip"));
    }

    if contains_ascii(bytes, "AndroidManifest.xml") {
        return high("apk", ClassificationBasis::Structure, Some("zip"));
    }

    medium("zip", ClassificationBasis::MagicBytes, None)
}

fn is_tiff(bytes: &[u8]) -> bool {
    bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*")
}

fn is_ico(bytes: &[u8]) -> bool {
    bytes.len() >= 8 && bytes[0..4] == [0, 0, 1, 0] && bytes[4] > 0
}

fn is_mp3(bytes: &[u8]) -> bool {
    if bytes.starts_with(b"ID3") {
        return true;
    }
    if bytes.len() < 2 {
        return false;
    }
    bytes[0] == 0xFF && (bytes[1] & 0xE0) == 0xE0
}

fn is_mkv_or_webm(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x1A, 0x45, 0xDF, 0xA3])
}

fn is_pe(bytes: &[u8]) -> bool {
    if !bytes.starts_with(b"MZ") || bytes.len() < 0x40 {
        return false;
    }
    let pe_offset =
        u32::from_le_bytes([bytes[0x3C], bytes[0x3D], bytes[0x3E], bytes[0x3F]]) as usize;
    if pe_offset.checked_add(4).is_none_or(|end| end > bytes.len()) {
        return false;
    }
    bytes[pe_offset..pe_offset + 4] == *b"PE\0\0"
}

fn is_macho(bytes: &[u8]) -> bool {
    let magics: [[u8; 4]; 6] = [
        [0xFE, 0xED, 0xFA, 0xCE],
        [0xFE, 0xED, 0xFA, 0xCF],
        [0xCE, 0xFA, 0xED, 0xFE],
        [0xCF, 0xFA, 0xED, 0xFE],
        [0xCA, 0xFE, 0xBA, 0xBE],
        [0xBE, 0xBA, 0xFE, 0xCA],
    ];
    magics.iter().any(|magic| bytes.starts_with(magic))
}

fn contains_ascii(bytes: &[u8], needle: &str) -> bool {
    bytes
        .windows(needle.len())
        .any(|window| window == needle.as_bytes())
}

fn looks_like_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }

    let printable = bytes
        .iter()
        .filter(|b| matches!(**b, b'\n' | b'\r' | b'\t' | 0x20..=0x7E))
        .count();
    printable * 100 / bytes.len() >= 90
}

fn looks_like_xml(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let trimmed = text.trim_start();
    trimmed.starts_with("<?xml") || (trimmed.starts_with('<') && trimmed.contains("</"))
}

fn looks_like_html(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let lowercase = text.to_ascii_lowercase();
    lowercase.contains("<html") || lowercase.contains("<!doctype html")
}

fn preferred_extension(classification: &ContentClassification) -> &'static str {
    MAGIC_RULES
        .iter()
        .find(|rule| rule.kind == classification.kind)
        .map(|rule| rule.ext)
        .unwrap_or_else(|| match classification.kind.as_str() {
            "docx" => "docx",
            "xlsx" => "xlsx",
            "pptx" => "pptx",
            "odt" => "odt",
            "ods" => "ods",
            "odp" => "odp",
            "epub" => "epub",
            "jar" => "jar",
            "apk" => "apk",
            "wav" => "wav",
            "avi" => "avi",
            "webp" => "webp",
            "aac" => "aac",
            "iso" => "iso",
            "tar" => "tar",
            "ico" => "ico",
            "tiff" => "tiff",
            "pe" => "exe",
            "mach_o" => "macho",
            "deb" => "deb",
            _ => "bin",
        })
}

fn probable_label(classification: &ContentClassification) -> String {
    classification
        .kind
        .replace(['_', ' '], "-")
        .to_ascii_lowercase()
}

fn high(
    kind: &'static str,
    basis: ClassificationBasis,
    subtype: Option<&'static str>,
) -> ContentClassification {
    ContentClassification {
        kind: kind.to_string(),
        confidence: RecoveryConfidence::High,
        basis,
        subtype: subtype.map(ToString::to_string),
    }
}

fn medium(
    kind: &'static str,
    basis: ClassificationBasis,
    subtype: Option<&'static str>,
) -> ContentClassification {
    ContentClassification {
        kind: kind.to_string(),
        confidence: RecoveryConfidence::Medium,
        basis,
        subtype: subtype.map(ToString::to_string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_parse_is_high_confidence() {
        let out = classify_and_name(br#"{"hello":"world"}"#, 1);
        assert!(matches!(
            out.classification.confidence,
            RecoveryConfidence::High
        ));
        assert_eq!(out.classification.kind, "json");
        assert_eq!(out.assigned_name, "file_000001.bin");
    }

    #[test]
    fn generic_zip_stays_medium() {
        let mut bytes = b"PK\x03\x04".to_vec();
        bytes.extend_from_slice(b"random zip body without subtype markers");
        let out = classify_and_name(&bytes, 2);
        assert!(matches!(
            out.classification.confidence,
            RecoveryConfidence::Medium
        ));
        assert_eq!(out.classification.kind, "zip");
        assert_eq!(out.assigned_name, "file_000002.probable-zip.bin");
    }

    #[test]
    fn docx_requires_zip_structure_markers() {
        let mut bytes = b"PK\x03\x04".to_vec();
        bytes.extend_from_slice(b"[Content_Types].xml word/document.xml");
        let out = classify_and_name(&bytes, 3);
        assert!(matches!(
            out.classification.confidence,
            RecoveryConfidence::High
        ));
        assert_eq!(out.classification.kind, "docx");
        assert_eq!(out.assigned_name, "file_000003.docx");
    }

    #[test]
    fn unknown_binary_is_low_confidence() {
        let bytes = [0x01, 0x02, 0x03, 0xFE, 0xFF, 0x00];
        let out = classify_and_name(&bytes, 7);
        assert!(matches!(
            out.classification.confidence,
            RecoveryConfidence::Low
        ));
        assert_eq!(out.classification.kind, "bin");
        assert_eq!(out.assigned_name, "file_000007.bin");
    }

    #[test]
    fn pe_uses_secondary_header_validation() {
        let mut bytes = vec![0u8; 256];
        bytes[0..2].copy_from_slice(b"MZ");
        bytes[0x3C..0x40].copy_from_slice(&(0x80u32).to_le_bytes());
        bytes[0x80..0x84].copy_from_slice(b"PE\0\0");

        let out = classify_and_name(&bytes, 9);
        assert!(matches!(
            out.classification.confidence,
            RecoveryConfidence::High
        ));
        assert_eq!(out.classification.kind, "pe");
        assert_eq!(out.assigned_name, "file_000009.exe");
    }
}
