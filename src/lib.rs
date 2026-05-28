//! Shift BED coordinates by a fixed offset.
//!
//! Each record's start and end are shifted by `shift` (positive = right,
//! negative = left). When a genome-sizes file is provided, coordinates are
//! clamped to `[0, chrom_size)`. Records that shift entirely off a chromosome
//! end are dropped (matching bedtools shift behaviour).
//!
//! Strand-aware mode (`-p`/`-m`): shift by `plus_shift` when strand column is
//! `+`, by `minus_shift` when `-`, else by the general `shift`.
//!
//! BED header/track/browser lines pass through unchanged.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

/// Load genome sizes file into a `chrom → size` map.
/// Each non-comment line is `chrom<TAB>size`.
pub fn load_genome(path: &Path) -> Result<HashMap<String, u64>> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let mut map = HashMap::new();
    for line in raw.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, '\t');
        let chrom = parts
            .next()
            .ok_or_else(|| RsomicsError::InvalidInput(format!("genome: bad line: {line:?}")))?;
        let size_str = parts
            .next()
            .ok_or_else(|| {
                RsomicsError::InvalidInput(format!("genome: missing size for {chrom}"))
            })?
            .trim();
        let size: u64 = size_str.parse().map_err(|_| {
            RsomicsError::InvalidInput(format!("genome: bad size {size_str:?} for {chrom}"))
        })?;
        map.insert(chrom.to_owned(), size);
    }
    Ok(map)
}

#[inline]
fn is_header(line: &str) -> bool {
    line.is_empty()
        || line.starts_with('#')
        || line.starts_with("track")
        || line.starts_with("browser")
}

/// Shift BED records from `reader` by `shift` bases, write to `output`.
///
/// `plus_shift` / `minus_shift` — per-strand overrides; set equal to `shift`
///   to disable strand-aware mode.
/// `genome` — when `Some`, coordinates are clamped and off-end records dropped.
pub fn shift_reader<R: io::Read>(
    reader: BufReader<R>,
    shift: i64,
    plus_shift: i64,
    minus_shift: i64,
    genome: Option<&HashMap<String, u64>>,
    output: &mut dyn Write,
) -> Result<()> {
    let mut out = BufWriter::with_capacity(256 * 1024, output);
    for (lineno_0, line) in reader.lines().enumerate() {
        let line = line.map_err(RsomicsError::Io)?;
        if is_header(&line) {
            out.write_all(line.as_bytes()).map_err(RsomicsError::Io)?;
            out.write_all(b"\n").map_err(RsomicsError::Io)?;
            continue;
        }
        let lineno = lineno_0 + 1;

        // Split into at most 6 columns; columns beyond 5 stay joined in col[5].
        let cols: Vec<&str> = line.splitn(7, '\t').collect();
        if cols.len() < 3 {
            return Err(RsomicsError::InvalidInput(format!(
                "line {lineno}: fewer than 3 fields"
            )));
        }
        let chrom = cols[0];
        let start: i64 = cols[1].parse().map_err(|_| {
            RsomicsError::InvalidInput(format!("line {lineno}: bad start {:?}", cols[1]))
        })?;
        let end: i64 = cols[2].parse().map_err(|_| {
            RsomicsError::InvalidInput(format!("line {lineno}: bad end {:?}", cols[2]))
        })?;

        // Column 5 (0-indexed 4) is the strand column in BED6+.
        let strand = if cols.len() >= 6 { cols[5] } else { "." };
        let s = if strand == "+" {
            plus_shift
        } else if strand == "-" {
            minus_shift
        } else {
            shift
        };

        let new_start = start + s;
        let new_end = end + s;

        let (new_start, new_end) = if let Some(g) = genome {
            let chrom_sz = g.get(chrom).copied().unwrap_or(i64::MAX as u64) as i64;
            let cs = new_start.max(0);
            let ce = new_end.min(chrom_sz);
            if cs >= ce {
                continue;
            }
            (cs, ce)
        } else {
            (new_start, new_end)
        };

        // Reconstruct output: first 3 cols updated, rest pass through verbatim.
        out.write_all(chrom.as_bytes()).map_err(RsomicsError::Io)?;
        write!(out, "\t{new_start}\t{new_end}").map_err(RsomicsError::Io)?;
        for col in &cols[3..] {
            out.write_all(b"\t").map_err(RsomicsError::Io)?;
            out.write_all(col.as_bytes()).map_err(RsomicsError::Io)?;
        }
        out.write_all(b"\n").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(())
}

/// Shift BED file at `path`.
pub fn shift(
    path: &Path,
    shift: i64,
    plus_shift: i64,
    minus_shift: i64,
    genome: Option<&HashMap<String, u64>>,
    output: &mut dyn Write,
) -> Result<()> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    shift_reader(
        BufReader::new(file),
        shift,
        plus_shift,
        minus_shift,
        genome,
        output,
    )
}

/// Same as [`shift`] but reads from stdin.
pub fn shift_stdin(
    shift_val: i64,
    plus_shift: i64,
    minus_shift: i64,
    genome: Option<&HashMap<String, u64>>,
    output: &mut dyn Write,
) -> Result<()> {
    shift_reader(
        BufReader::new(io::stdin()),
        shift_val,
        plus_shift,
        minus_shift,
        genome,
        output,
    )
}
