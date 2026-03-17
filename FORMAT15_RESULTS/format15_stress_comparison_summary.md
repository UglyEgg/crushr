# Format-15 stress comparison

## Judgment

1. Does namespace factoring materially reduce dictionary size? **No**.
2. Does the factored mirrored dictionary variant remain smaller than inline-path identity? **Yes**.
3. Does generation-aware identity improve dictionary conflict semantics? **No**.
4. Can one valid mirrored copy still preserve named recovery when the other is invalid? **Yes**.
5. Does the factored mirrored dictionary variant now become the preferred canonical candidate? **Yes**.
6. Is the added structural complexity justified by the measured size savings? **No**.
