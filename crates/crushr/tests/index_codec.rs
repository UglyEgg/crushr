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
                    logical_offset: 0,
                }],
                link_target: None,
                xattrs: Vec::new(),
                uid: 1000,
                gid: 1000,
                uname: Some("user".to_string()),
                gname: Some("group".to_string()),
                hardlink_group_id: None,
                sparse: false,
                device_major: None,
                device_minor: None,
                acl_access: Some(vec![1, 2, 3]),
                acl_default: None,
                selinux_label: Some(b"system_u:object_r:tmp_t:s0".to_vec()),
                linux_capability: Some(vec![0x01, 0, 0, 0]),
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
                    logical_offset: 0,
                }],
                link_target: None,
                xattrs: Vec::new(),
                uid: 1000,
                gid: 1000,
                uname: None,
                gname: None,
                hardlink_group_id: Some(42),
                sparse: false,
                device_major: None,
                device_minor: None,
                acl_access: None,
                acl_default: None,
                selinux_label: None,
                linux_capability: None,
            },
        ],
    };

    let bytes = encode_index(&idx);
    let dec = decode_index(&bytes).unwrap();
    assert_eq!(dec.entries.len(), 2);
    assert_eq!(dec.entries[0].path, "a.txt");
    assert_eq!(dec.entries[1].path, "sub/b.json");
    assert_eq!(dec.entries[1].extents[0].block_id, 1);
    assert_eq!(dec.entries[0].acl_access, Some(vec![1, 2, 3]));
    assert_eq!(
        dec.entries[0].selinux_label,
        Some(b"system_u:object_r:tmp_t:s0".to_vec())
    );
    assert_eq!(dec.entries[0].linux_capability, Some(vec![0x01, 0, 0, 0]));
}
