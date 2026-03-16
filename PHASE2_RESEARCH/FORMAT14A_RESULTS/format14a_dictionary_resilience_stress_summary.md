# Format-14A dictionary resilience stress comparison

## Judgment

1. Is `extent_identity_path_dict_single` too fragile under direct dictionary-target corruption? **Yes** (named recovery with primary dictionary loss = 0).
2. Does `extent_identity_path_dict_header_tail` preserve named recovery when one dictionary copy is lost? **Yes** (named recovery with mirror loss = 4).
3. When both dictionary copies are lost, does salvage fail closed for naming and fall back to anonymous recovery correctly? **Yes** (anonymous fallback count = 4).
4. Are conflicting surviving dictionary copies detected and handled safely? **Yes** (conflicts detected = 4, fail-closed = 4).
5. Which dictionary placement strategy should remain the lead candidate going forward? **extent_identity_path_dict_header_tail**.
