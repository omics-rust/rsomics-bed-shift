# rsomics-bed-shift

Shift BED coordinates by a fixed offset — `bedtools shift` equivalent in pure Rust.

## Install

```sh
cargo install rsomics-bed-shift
```

## Usage

```sh
rsomics-bed-shift -s 100 input.bed                          # shift all features 100 bp downstream
rsomics-bed-shift -s 500 -g hg38.genome input.bed           # clamp to chromosome boundaries
rsomics-bed-shift -p 100 -m -100 input.bed                  # strand-aware shift
rsomics-bed-shift -s -50 -g hg38.genome < input.bed         # stdin
```

## Options

| Flag | Description |
|---|---|
| `-s <INT>` | Bases to shift (positive = downstream, negative = upstream; default: 0) |
| `-p <INT>` | Per-strand override: shift for `+` strand features |
| `-m <INT>` | Per-strand override: shift for `-` strand features |
| `-g <FILE>` | Genome sizes file (`chrom<TAB>size`); clamps coordinates and drops off-end records |
| `--out <FILE>` | Output path (default: stdout) |

## Origin

This crate is an independent Rust reimplementation of `bedtools shift` based on:
- The [bedtools2 documentation](https://bedtools.readthedocs.io/en/latest/content/tools/shift.html)
- The BED format specification
- Black-box behaviour testing against `bedtools shift`

License: MIT OR Apache-2.0  
Upstream credit: [bedtools2](https://github.com/arq5x/bedtools2) (MIT)
