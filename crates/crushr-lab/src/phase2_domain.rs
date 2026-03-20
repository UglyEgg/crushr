// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use serde::{Deserialize, Serialize};

pub const PHASE2_SCENARIO_ID_FORMAT: &str =
    "p2-core-{dataset}-{format_id}-{corruption_type}-{target_class}-{magnitude}-{seed}";
pub const LOCKED_CORE_SEEDS: [u64; 3] = [1337, 2600, 65535];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dataset {
    Smallfiles,
    Mixed,
    Largefiles,
}

impl Dataset {
    pub fn ordered_locked_core() -> &'static [Self] {
        &[Self::Smallfiles, Self::Mixed, Self::Largefiles]
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Smallfiles => "smallfiles",
            Self::Mixed => "mixed",
            Self::Largefiles => "largefiles",
        }
    }

    pub fn composition_rule(self) -> &'static str {
        match self {
            Self::Smallfiles => {
                "24 UTF-8 text files split across 6 folders; file i has exactly i+3 lines with deterministic sentence payload"
            }
            Self::Mixed => {
                "12 text files + 4 deterministic binary blobs + 2 JSON files + 2 CSV files in stable nested layout"
            }
            Self::Largefiles => {
                "3 larger files (2 binary, 1 text) with deterministic byte/line generation and stable sizes"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArchiveFormat {
    #[serde(rename = "crushr")]
    Crushr,
    #[serde(rename = "zip")]
    Zip,
    #[serde(rename = "tar+zstd")]
    TarZstd,
    #[serde(rename = "tar+gz")]
    TarGz,
    #[serde(rename = "tar+xz")]
    TarXz,
}

impl ArchiveFormat {
    pub fn ordered_locked_core() -> &'static [Self] {
        &[
            Self::Crushr,
            Self::Zip,
            Self::TarZstd,
            Self::TarGz,
            Self::TarXz,
        ]
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Crushr => "crushr",
            Self::Zip => "zip",
            Self::TarZstd => "tar_zstd",
            Self::TarGz => "tar_gz",
            Self::TarXz => "tar_xz",
        }
    }

    pub fn output_file_name(self, dataset: Dataset) -> String {
        match self {
            Self::Crushr => format!("{}.crs", dataset.slug()),
            Self::Zip => format!("{}.zip", dataset.slug()),
            Self::TarZstd => format!("{}.tar.zst", dataset.slug()),
            Self::TarGz => format!("{}.tar.gz", dataset.slug()),
            Self::TarXz => format!("{}.tar.xz", dataset.slug()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorruptionType {
    BitFlip,
    ByteOverwrite,
    ZeroFill,
    Truncation,
    TailDamage,
}

impl CorruptionType {
    pub fn ordered_locked_core() -> &'static [Self] {
        &[
            Self::BitFlip,
            Self::ByteOverwrite,
            Self::ZeroFill,
            Self::Truncation,
            Self::TailDamage,
        ]
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::BitFlip => "bit_flip",
            Self::ByteOverwrite => "byte_overwrite",
            Self::ZeroFill => "zero_fill",
            Self::Truncation => "truncation",
            Self::TailDamage => "tail_damage",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetClass {
    Header,
    Index,
    Payload,
    Tail,
}

impl TargetClass {
    pub fn ordered_locked_core() -> &'static [Self] {
        &[Self::Header, Self::Index, Self::Payload, Self::Tail]
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Index => "index",
            Self::Payload => "payload",
            Self::Tail => "tail",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Magnitude {
    #[serde(rename = "1B")]
    OneByte,
    #[serde(rename = "256B")]
    TwoHundredFiftySixBytes,
    #[serde(rename = "4KB")]
    FourKilobytes,
}

impl Magnitude {
    pub fn bytes(self) -> u64 {
        match self {
            Self::OneByte => 1,
            Self::TwoHundredFiftySixBytes => 256,
            Self::FourKilobytes => 4096,
        }
    }

    pub fn ordered_locked_core() -> &'static [Self] {
        &[
            Self::OneByte,
            Self::TwoHundredFiftySixBytes,
            Self::FourKilobytes,
        ]
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::OneByte => "1B",
            Self::TwoHundredFiftySixBytes => "256B",
            Self::FourKilobytes => "4KB",
        }
    }
}

pub fn locked_core_scenario_id(
    dataset: Dataset,
    format: ArchiveFormat,
    corruption_type: CorruptionType,
    target_class: TargetClass,
    magnitude: Magnitude,
    seed: u64,
) -> String {
    format!(
        "p2-core-{}-{}-{}-{}-{}-{}",
        dataset.slug(),
        format.slug(),
        corruption_type.slug(),
        target_class.slug(),
        magnitude.slug(),
        seed
    )
}
