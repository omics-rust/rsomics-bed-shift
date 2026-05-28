use std::io;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bed_shift::{load_genome, shift, shift_stdin};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-bed-shift", disable_help_flag = true)]
pub struct Cli {
    /// Input BED file (default: stdin)
    input: Option<PathBuf>,
    /// Number of bases to shift coordinates (positive = right, negative = left)
    #[arg(short = 's', long, default_value = "0", allow_hyphen_values = true)]
    shift: i64,
    /// Shift to apply to features on the + strand (strand-aware mode)
    #[arg(short = 'p', long, allow_hyphen_values = true)]
    plus_shift: Option<i64>,
    /// Shift to apply to features on the - strand (strand-aware mode)
    #[arg(short = 'm', long, allow_hyphen_values = true)]
    minus_shift: Option<i64>,
    /// Genome sizes file (chrom<TAB>size); enables coordinate clamping
    #[arg(short = 'g', long)]
    genome: Option<PathBuf>,
    /// Output path (default: stdout)
    #[arg(long = "out")]
    output: Option<PathBuf>,
    #[command(flatten)]
    pub common: CommonFlags,
}

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }
    fn common(&self) -> &CommonFlags {
        &self.common
    }

    fn execute(self) -> Result<()> {
        let plus_shift = self.plus_shift.unwrap_or(self.shift);
        let minus_shift = self.minus_shift.unwrap_or(self.shift);

        let genome = if let Some(ref gpath) = self.genome {
            Some(load_genome(gpath)?)
        } else {
            None
        };

        let mut stdout_lock;
        let mut file_out;
        let out: &mut dyn io::Write = if let Some(ref p) = self.output {
            file_out = std::fs::File::create(p).map_err(RsomicsError::Io)?;
            &mut file_out
        } else {
            stdout_lock = io::stdout().lock();
            &mut stdout_lock
        };

        match self.input {
            Some(ref p) => shift(p, self.shift, plus_shift, minus_shift, genome.as_ref(), out),
            None => shift_stdin(self.shift, plus_shift, minus_shift, genome.as_ref(), out),
        }
    }
}

pub const HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Shift BED coordinates by a fixed offset.",
    origin: Some(Origin {
        upstream: "bedtools",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btq033"),
    }),
    usage_lines: &["-s <N> [OPTIONS] [INPUT]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('s'),
                long: "shift",
                aliases: &[],
                value: Some("<INT>"),
                type_hint: Some("i64"),
                required: false,
                default: Some("0"),
                description: "Bases to shift (positive = right/downstream, negative = left/upstream)",
                why_default: None,
            },
            FlagSpec {
                short: Some('p'),
                long: "plus-shift",
                aliases: &[],
                value: Some("<INT>"),
                type_hint: Some("i64"),
                required: false,
                default: None,
                description: "Shift for + strand features (enables strand-aware mode)",
                why_default: None,
            },
            FlagSpec {
                short: Some('m'),
                long: "minus-shift",
                aliases: &[],
                value: Some("<INT>"),
                type_hint: Some("i64"),
                required: false,
                default: None,
                description: "Shift for - strand features (enables strand-aware mode)",
                why_default: None,
            },
            FlagSpec {
                short: Some('g'),
                long: "genome",
                aliases: &[],
                value: Some("<FILE>"),
                type_hint: Some("Path"),
                required: false,
                default: None,
                description: "Genome sizes file; coordinates clamped to [0, chrom_size)",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "out",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: false,
                default: Some("stdout"),
                description: "Output path",
                why_default: None,
            },
            FlagSpec {
                short: Some('h'),
                long: "help",
                aliases: &[],
                value: None,
                type_hint: Some("bool"),
                required: false,
                default: None,
                description: "Show this help",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Shift all features 100 bp downstream",
            command: "rsomics-bed-shift -s 100 input.bed",
        },
        Example {
            description: "Shift with genome clamping (drop off-end features)",
            command: "rsomics-bed-shift -s 500 -g hg38.genome input.bed",
        },
        Example {
            description: "Strand-aware shift: +100 on + strand, -100 on - strand",
            command: "rsomics-bed-shift -p 100 -m -100 input.bed",
        },
    ],
    json_result_schema_doc: None,
};

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    #[test]
    fn cli_definition_is_valid() {
        super::Cli::command().debug_assert();
    }
}
