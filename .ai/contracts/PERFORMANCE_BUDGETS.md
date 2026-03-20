<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Performance Budgets

Initial budgets are guidance, not promises.

- verification should scale linearly with blocks examined
- impact enumeration should be `O(total_extents)` without decompression
- metadata overhead should remain small relative to block size
- performance regressions must be measured before format/integrity guarantees are weakened
