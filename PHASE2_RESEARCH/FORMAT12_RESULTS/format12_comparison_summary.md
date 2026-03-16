# Format-12 inline path comparison

## Variant summary

| variant | scenarios | named | anon_full | partial_ordered | partial_unordered | orphan | none | archive_byte_size | overhead_vs_payload_only | overhead_vs_extent_identity_only | overhead_vs_payload_plus_manifest | named_delta_vs_extent_identity_only | named_delta_vs_payload_plus_manifest | recovery_per_kib_overhead |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| payload_only | 24 | 2 | 10 | 0 | 0 | 0 | 0 | 51808 | 0 | -192 | -35080 | 0 | -10 | 0.000000 |
| extent_identity_only | 24 | 2 | 10 | 0 | 0 | 0 | 0 | 52000 | 192 | 0 | -34888 | 0 | -10 | 0.000000 |
| extent_identity_distributed_names | 24 | 12 | 0 | 0 | 0 | 0 | 0 | 75312 | 23504 | 23312 | -11576 | 10 | 0 | 0.439259 |
| payload_plus_manifest | 24 | 12 | 0 | 0 | 0 | 0 | 0 | 86888 | 35080 | 34888 | 0 | 10 | 0 | 0.293511 |
| full_current_experimental | 24 | 12 | 0 | 0 | 0 | 0 | 0 | 109984 | 58176 | 57984 | 23096 | 10 | 0 | 0.176600 |
| extent_identity_inline_path | 24 | 12 | 0 | 0 | 0 | 0 | 0 | 54280 | 2472 | 2280 | -32608 | 10 | 0 | 4.491228 |

## Explicit judgment

- `extent_identity_inline_path` is credible for compression-oriented use only if its overhead remains materially below `payload_plus_manifest` while preserving similar named recovery.
- Use the table above to determine whether overhead is closer to `extent_identity_only`, `payload_plus_manifest`, or `full_current_experimental`.
- If named-recovery gain is small relative to added bytes, this variant should be treated as evidence for a more compact distributed naming design rather than immediate adoption.
