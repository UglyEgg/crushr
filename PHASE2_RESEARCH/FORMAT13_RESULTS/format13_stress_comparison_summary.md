# Format-13 stress comparison

## Judgment

1. Does dictionary encoding reduce archive size relative to `extent_identity_inline_path`? **See per-variant overhead columns.**
2. Which placement strategy preserves named recovery best under header, index, and tail corruption? **Compare the three successful_named_recovery_with_* metrics.**
3. Is a single header dictionary too fragile? **If `extent_identity_path_dict_single` drops named recovery under header corruption, yes.**
4. Is header+tail sufficient? **Use header+tail vs quasi-uniform named-recovery deltas.**
5. Does quasi-uniform interior mirroring materially improve resilience? **Use named-recovery and corruption-target grouped breakdown.**
6. Which dictionary placement strategy is the best next-step candidate? **Choose the best named-recovery/overhead tradeoff in this summary.**
7. Does the winning dictionary strategy surpass inline path strongly enough to justify replacing it as the lead candidate? **Decide from overhead_delta_vs_extent_identity_inline_path plus recovery parity.**
