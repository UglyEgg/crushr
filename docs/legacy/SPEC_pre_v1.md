# crushr archive format (MVP)

## Goals
- **Small** archives on typical "many file" inputs
- **Practical** CPU-only encode time
- **Seekable** random access when the archive is a regular file
- **Simple** format that can evolve

Non-goals (MVP):
- perfect streaming support over stdin
- cryptographic immutability ("can't flip RO")
- optimal single-file extraction speed

## File layout

```
[Block 0][Block 1]...[Block N][Index][Footer]
```

### Blocks
Each block stores a compressed payload representing a segment of the solid, concatenated file-data stream.

Block header (little-endian):
- magic: 4 bytes = `BLK1`
- codec: u32 (1 = zstd)
- level: i32 (zstd level used)
- uncompressed_size: u64
- compressed_size: u64
- payload_hash: 32 bytes (BLAKE3 of **compressed payload bytes**)

Then:
- payload: `compressed_size` bytes

### Index
The index bytes start with magic `IDX3` (legacy `IDX1`/`IDX2` supported for read) and encode a file table in little-endian form:

- magic: 4 bytes = `IDX3`
- entry_count: u32
- for each entry:
  - path_len: u32, then UTF-8 path bytes
  - mode: u32
  - mtime: i64 (unix seconds)
  - size: u64
  - extent_count: u32
  - for each extent:
    - block_id: u32
    - offset: u64 (within uncompressed block)
    - len: u64

For `IDX1`, trailing bytes are not permitted (exact consumption required).

### Footer
Footer is fixed-size and always at end-of-file.

Footer layout (current):
- magic: 4 bytes = `FTR2`
- blocks_end_offset: u64
- index_offset: u64
- index_len: u64
- index_hash: 32 bytes (BLAKE3 of index bytes)

Legacy footer (read supported):
- magic: 4 bytes = `FTR1`
- index_offset: u64
- index_len: u64
- index_hash: 32 bytes (BLAKE3 of index bytes)

## Extraction model
- Seek to footer, read index offset/len
- Load index, verify `index_hash`
- For a file, for each extent:
  - read block header + payload
  - verify payload hash
  - zstd decompress payload
  - copy the requested slice for that extent

MVP extracts by fully decompressing each referenced block (no cache).

## Mutability model (planned)
- Append: write more blocks, then write a new index+footer
- Delete: write tombstone records and update index; reclaim with compact
- RO: optional "seal" via signature over footer+index (future)

## Versioning
- MVP uses `BLK1` and `FTR1` magic; future versions can introduce `BLK2`/`FTR2` with compatibility rules.


## Append semantics (current)
- The archive is mutable by rewriting the tail: truncate at `blocks_end_offset`, append new blocks, then write a new index and footer.
- Readers always use the *last* footer at end-of-file.


## Path normalization
- Stored paths are UTF-8 strings with `/` separators.
- CLI uses `--base` to compute stored paths by stripping the base prefix.
- For safety, inputs not under `--base` are rejected.


## Dictionary support
- New archives write `BLK2` blocks when a dictionary is used.
- `BLK2` header adds `dict_id: u32` (currently 1 = archive dictionary).
- New footer `FTR3` adds a dictionary region pointer:
  - blocks_end_offset: u64
  - dict_offset: u64
  - dict_len: u64
  - index_offset: u64
  - index_len: u64
  - index_hash: [u8;32]
- The dictionary bytes are stored verbatim at `dict_offset`.


### Dictionary table (DCT1)
- `FTR3`'s `dict_offset/dict_len` points to a dictionary table, not raw dict bytes.
- Dictionary table encoding:
  - magic: `DCT1`
  - version: u32 = 1
  - count: u32
  - repeated entries:
    - dict_id: u32
    - dict_hash: [u8;32] (BLAKE3 of raw dict bytes)
    - dict_len: u32
    - dict_bytes: [u8;dict_len]
- `BLK2` stores `dict_id` for the block (0 = no dict).


## Default dictionary families
- `dict_id=1`: Text family (md/txt/json/yaml/toml/...)
- `dict_id=2`: Code family (rs/py/c/h/cpp/js/ts/...)
- `dict_id=0`: no dictionary


## Tail redundancy
- Writers append a backup copy of the index bytes followed by a second `FTR3` footer pointing to that backup index.
- Readers normally use the last footer at EOF.
- `recover` can scan the tail to locate an earlier valid footer and rewrite a healthy archive.


### Tail frames
- Writers may append multiple redundant tail frames:
  - optional padding bytes
  - index bytes
  - `FTR3` pointing at that index copy
- Readers use the last valid `FTR3` at EOF.

## Embedded event frames (EVT1)
To enable indexless salvage, writers may embed EVT frames in the blocks region.

Frame encoding:
- magic: `EVT1`
- kind: u32
  - 1 = file events
  - 2 = dictionary table bytes (`DCT1` payload)
- payload_len: u32
- payload_hash: [u8;32] = BLAKE3(payload)
- payload bytes

File events payload:
- count: u32
- repeated:
  - block_id: u32 (data block ordinal)
  - intra: u32 (offset within the uncompressed block where file data begins)
  - kind: u8 (0 regular, 1 symlink)
  - mode: u32
  - mtime: i64
  - size: u64
  - path: len+bytes
  - link_target: len+bytes (0 for regular)

These frames are ignored by normal readers (index-driven) but can be used by `salvage` to rebuild an index.
