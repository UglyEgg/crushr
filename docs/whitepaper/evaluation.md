<!--
SPDX-License-Identifier: CC-BY-4.0
SPDX-FileCopyrightText: 2026 Richard Majewski
-->

# Experimental Evaluation

crushr architecture was selected through destructive testing.

## Method

- generate archive
- apply deterministic corruption
- measure recovery

## Metrics

| Metric | Meaning |
|-------|--------|
| recovery_rate | % of data recovered |
| name_retention | % of filenames preserved |
| false_positive | incorrect recovery |

## Result summary

- distributed identity consistently outperformed central metadata
- mirrored dictionaries retained names under partial corruption
- FORMAT-15 failed due to leadership dependency

## Conclusion

Architecture is evidence-driven, not theoretical.
