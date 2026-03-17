# Security Model

crushr processes untrusted archive bytes and may write files to disk.

Requirements:
- all parsers are length-bounded and reject malformed lengths
- extraction must defend against path traversal and unsafe symlink restoration
- corruption analysis must never require executing arbitrary content
- fuzzing of parsers is a required hardening activity, not a future nice-to-have
