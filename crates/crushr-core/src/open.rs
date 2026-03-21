// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crate::io::{Len, ReadAt};
use anyhow::{Context, Result, ensure};
use crushr_format::ftr4::{FTR4_LEN, Ftr4};
use crushr_format::tailframe::{TailFrameParts, parse_tail_frame};
use std::io::Cursor;

#[derive(Debug, Clone)]
pub struct OpenArchiveV1 {
    pub archive_len: u64,
    pub tail_frame_offset: u64,
    pub tail_frame_len: u64,
    pub footer_offset: u64,
    pub footer_len: u64,
    pub tail: TailFrameParts,
}

pub fn open_archive_v1<R: ReadAt + Len>(reader: &R) -> Result<OpenArchiveV1> {
    let archive_len = reader.len().context("read archive length")?;
    ensure!(
        archive_len >= FTR4_LEN as u64,
        "archive too short to contain FTR4"
    );

    let footer_offset = archive_len - (FTR4_LEN as u64);
    let mut footer_bytes = vec![0u8; FTR4_LEN];
    read_exact_at(reader, footer_offset, &mut footer_bytes).context("read FTR4")?;
    let footer = Ftr4::read_from(Cursor::new(&footer_bytes)).context("parse FTR4")?;

    ensure!(
        footer.blocks_end_offset <= footer_offset,
        "tail frame start is after footer start"
    );

    let tail_frame_offset = footer.blocks_end_offset;
    let tail_frame_len = archive_len
        .checked_sub(tail_frame_offset)
        .context("tail frame length underflow")?;

    let mut tail_frame_bytes = vec![0u8; tail_frame_len as usize];
    read_exact_at(reader, tail_frame_offset, &mut tail_frame_bytes)
        .context("read tail frame bytes")?;

    let tail = parse_tail_frame(&tail_frame_bytes).context("parse tail frame")?;

    Ok(OpenArchiveV1 {
        archive_len,
        tail_frame_offset,
        tail_frame_len,
        footer_offset,
        footer_len: FTR4_LEN as u64,
        tail,
    })
}

fn read_exact_at<R: ReadAt>(reader: &R, mut offset: u64, mut dst: &mut [u8]) -> Result<()> {
    while !dst.is_empty() {
        let read = reader.read_at(offset, dst)?;
        ensure!(read != 0, "unexpected EOF while reading archive");
        let (_, rest) = dst.split_at_mut(read);
        dst = rest;
        offset = offset
            .checked_add(read as u64)
            .context("offset overflow while reading archive")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::{Len, ReadAt};
    use anyhow::Result;
    use crushr_format::tailframe::assemble_tail_frame;

    #[derive(Clone)]
    struct MemReader {
        bytes: Vec<u8>,
    }

    impl ReadAt for MemReader {
        fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
            let offset = offset as usize;
            if offset >= self.bytes.len() {
                return Ok(0);
            }
            let n = (self.bytes.len() - offset).min(buf.len());
            buf[..n].copy_from_slice(&self.bytes[offset..offset + n]);
            Ok(n)
        }
    }

    impl Len for MemReader {
        fn len(&self) -> Result<u64> {
            Ok(self.bytes.len() as u64)
        }
    }

    #[test]
    fn opens_archive_with_valid_tail_frame() {
        let idx3 = b"IDX3\x00\x01\x02";
        let mut bytes = vec![0u8; 16];
        let tail = assemble_tail_frame(16, None, idx3, None).unwrap();
        bytes.extend_from_slice(&tail);
        let reader = MemReader { bytes };

        let opened = open_archive_v1(&reader).unwrap();
        assert_eq!(opened.archive_len as usize, reader.bytes.len());
        assert_eq!(opened.footer_len, FTR4_LEN as u64);
        assert_eq!(opened.footer_offset + opened.footer_len, opened.archive_len);
        assert_eq!(opened.tail.footer.index_len, idx3.len() as u64);
        assert!(opened.tail.dct1.is_none());
        assert!(opened.tail.ldg1.is_none());
    }

    #[test]
    fn rejects_short_archive() {
        let reader = MemReader {
            bytes: vec![1, 2, 3],
        };
        assert!(open_archive_v1(&reader).is_err());
    }
}
