# Engineering Motivation

crushr grew out of discomfort with how poorly consumer archive formats communicate failure domains. Many formats can detect corruption, but few can enumerate the exact impact before extraction. That gap is small in marketing terms and significant in engineering terms.

The project is intentionally restrained:
- detect and isolate, not reconstruct
- explain, do not guess
- prefer explicit contracts to hidden cleverness

The goal is not to replace ubiquitous formats. The goal is to validate a structurally stronger design tradeoff and demonstrate serious systems engineering in a niche that is easy to dismiss until it fails in production.
