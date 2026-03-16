# Format-12 stress comparison

## Variant summary

| variant | scenarios | archive_byte_size | overhead_vs_payload_only | overhead_vs_extent_identity_only | named | anon_full | partial_ordered | partial_unordered | orphan | none | avg_path | max_path | total_extents | avg_extents_per_file | max_extents_per_file | bytes_added_per_extent_vs_extent_identity_only | bytes_added_per_path_character_vs_extent_identity_only |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| payload_only | 32 | 21895832 | 0 | -256 | 0 | 16 | 0 | 0 | 0 | 0 | 193.11 | 348 | 19520 | 24.50 | 48 | -0.013115 | -0.000068 |
| extent_identity_only | 32 | 21896088 | 256 | 0 | 0 | 16 | 0 | 0 | 0 | 0 | 193.11 | 348 | 19520 | 24.50 | 48 | 0.000000 | 0.000000 |
| extent_identity_inline_path | 32 | 24666456 | 2770624 | 2770368 | 16 | 0 | 0 | 0 | 0 | 0 | 193.11 | 348 | 19520 | 24.50 | 48 | 141.924590 | 0.731864 |
| payload_plus_manifest | 32 | 31518288 | 9622456 | 9622200 | 16 | 0 | 0 | 0 | 0 | 0 | 193.11 | 348 | 19520 | 24.50 | 48 | 492.940574 | 2.541951 |

## Judgment

1. Did inline naming overhead materially increase under stress? **Yes** (inline overhead vs extent_identity_only = 2770368 bytes across all stress scenarios).
2. Does `extent_identity_inline_path` remain much smaller than `payload_plus_manifest`? **Yes** (2770368 vs 9622200 bytes overhead vs extent_identity_only).
3. Does fragmentation multiply path duplication cost into unacceptable territory? **Possibly** (bytes added per extent vs extent_identity_only = 141.924590).
4. Is `extent_identity_inline_path` still credible for a compression-oriented archive format? **Yes, pending FORMAT-13 policy lock** (size/recovery tradeoff remains bounded in this stress run).
5. Should it remain the leading candidate going into FORMAT-13? **Yes** as the lead identity-layer baseline for the next packet decision.
