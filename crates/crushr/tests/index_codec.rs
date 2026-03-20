// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Richard Majewski

use crushr::format::{Entry, EntryKind, Extent, Index};
use crushr::index_codec::{decode_index, encode_index};

#[test]
fn idx1_roundtrip() {
    let idx = Index {
        entries: vec![
            Entry {
                path: "a.txt".to_string(),
                kind: EntryKind::Regular,
                mode: 0o100644,
                mtime: 1700000000,
                size: 5,
                extents: vec![Extent {
                    block_id: 0,
                    offset: 0,
                    len: 5,
                }],
                link_target: None,
                xattrs: Vec::new(),
            },
            Entry {
                path: "sub/b.json".to_string(),
                kind: EntryKind::Regular,
                mode: 0o100644,
                mtime: 1700000001,
                size: 10,
                extents: vec![Extent {
                    block_id: 1,
                    offset: 7,
                    len: 10,
                }],
                link_target: None,
                xattrs: Vec::new(),
            },
        ],
    };

    let bytes = encode_index(&idx);
    let dec = decode_index(&bytes).unwrap();
    assert_eq!(dec.entries.len(), 2);
    assert_eq!(dec.entries[0].path, "a.txt");
    assert_eq!(dec.entries[1].path, "sub/b.json");
    assert_eq!(dec.entries[1].extents[0].block_id, 1);
}
