# Phase 2 Cross-Format Comparison Summary

Total normalized runs: **2700**

## Survivability overview

| Format | Runs | Mean file recovery | Mean byte recovery | Success rate | Partial-or-better rate | Tool error rate |
|---|---|---|---|---|---|---|
| crushr | 540 | 0.000000 | 0.000000 | 0.338889 | 0.338889 | 0.214815 |
| zip | 540 | 0.608704 | 0.566580 | 0.050000 | 0.177778 | 0.666667 |
| tar+zstd | 540 | 0.338889 | 0.285527 | 0.003704 | 0.305556 | 0.451852 |
| tar+gz | 540 | 0.582222 | 0.605515 | 0.007407 | 0.642593 | 0.000000 |
| tar+xz | 540 | 0.591065 | 0.581106 | 0.001852 | 0.614815 | 0.159259 |

## Blast radius distribution

| Format | NONE | LOCALIZED | PARTIAL_SET | WIDESPREAD | TOTAL |
|---|---|---|---|---|---|
| crushr | 0 | 0 | 0 | 0 | 540 |
| zip | 210 | 71 | 62 | 23 | 174 |
| tar+zstd | 161 | 0 | 33 | 0 | 346 |
| tar+gz | 220 | 34 | 75 | 50 | 161 |
| tar+xz | 210 | 47 | 56 | 145 | 82 |

## Survivability ranking by mean file recovery

| Rank | Format | Mean file recovery |
|---|---|---|
| 1 | zip | 0.608704 |
| 2 | tar+xz | 0.591065 |
| 3 | tar+gz | 0.582222 |
| 4 | tar+zstd | 0.338889 |
| 5 | crushr | 0.000000 |

## Notes

- Rates are proportions in the closed interval [0, 1].
- Recovery metrics are derived from extracted file presence and byte counts.
- This summary does not reinterpret or modify normalized inputs.
