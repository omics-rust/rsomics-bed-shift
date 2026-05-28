use std::path::Path;

use rsomics_bed_shift::{load_genome, shift};

fn golden(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

#[test]
fn basic_positive_shift() {
    let input = golden("input.bed");
    let mut out = Vec::new();
    shift(&input, 100, 100, 100, None, &mut out).unwrap();
    let result = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = result.lines().filter(|l| !l.is_empty()).collect();
    // feat1: 100→200, 200→300
    assert!(
        lines[0].starts_with("chr1\t200\t300\t"),
        "feat1 shifted wrong: {}",
        lines[0]
    );
    // feat4 (3-col BED): 10→110, 50→150
    assert_eq!(
        lines[3], "chr1\t110\t150\tfeat4",
        "feat4 shifted wrong: {}",
        lines[3]
    );
}

#[test]
fn negative_shift() {
    let input = golden("input.bed");
    let mut out = Vec::new();
    shift(&input, -10, -10, -10, None, &mut out).unwrap();
    let result = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = result.lines().filter(|l| !l.is_empty()).collect();
    // feat1: 100-10=90, 200-10=190
    assert!(
        lines[0].starts_with("chr1\t90\t190\t"),
        "feat1 negative shift wrong: {}",
        lines[0]
    );
}

#[test]
fn genome_clamping_drops_off_end() {
    let input = golden("input.bed");
    let genome_path = golden("genome.txt");
    let genome = load_genome(&genome_path).unwrap();
    let mut out = Vec::new();
    // Shift +900: feat1 [100,200) → [1000,1100) — both off chr1 (max=1000), dropped
    shift(&input, 900, 900, 900, Some(&genome), &mut out).unwrap();
    let result = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = result.lines().filter(|l| !l.is_empty()).collect();
    // feat1 [1000,1100): clamped to [1000,1000) → dropped (start>=end)
    // feat2 [1400,1500): off chr1 → dropped
    // feat3 [950,1050): clamped to [950,1000) → kept
    // feat4 [910,950): kept
    assert!(
        lines.iter().all(|l| !l.contains("feat1")),
        "feat1 should be dropped: {result}"
    );
    assert!(
        lines.iter().any(|l| l.contains("feat3")),
        "feat3 should be kept: {result}"
    );
}

#[test]
fn strand_aware_shift() {
    let input = golden("input.bed");
    let mut out = Vec::new();
    // plus_shift=50, minus_shift=-50, general=0
    shift(&input, 0, 50, -50, None, &mut out).unwrap();
    let result = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = result.lines().filter(|l| !l.is_empty()).collect();
    // feat1 (+): 100+50=150, 200+50=250
    assert!(
        lines[0].starts_with("chr1\t150\t250\t"),
        "feat1 strand+ wrong: {}",
        lines[0]
    );
    // feat2 (-): 500-50=450, 600-50=550
    assert!(
        lines[1].starts_with("chr1\t450\t550\t"),
        "feat2 strand- wrong: {}",
        lines[1]
    );
    // feat3 (.): no strand, general shift=0
    assert!(
        lines[2].starts_with("chr2\t50\t150\t"),
        "feat3 no-strand wrong: {}",
        lines[2]
    );
}

#[test]
fn bedtools_compat() {
    use std::process::Command;
    let bedtools = Command::new("bedtools").arg("--version").output();
    if bedtools.is_err() || !bedtools.unwrap().status.success() {
        eprintln!("bedtools not available — skipping compat test");
        return;
    }

    let input = golden("input.bed");
    let genome_path = golden("genome.txt");
    let genome = load_genome(&genome_path).unwrap();
    let mut ours = Vec::new();
    shift(&input, 100, 100, 100, Some(&genome), &mut ours).unwrap();
    let ours_str = String::from_utf8(ours).unwrap();

    let bt = Command::new("bedtools")
        .args(["shift", "-i"])
        .arg(&input)
        .arg("-s")
        .arg("100")
        .arg("-g")
        .arg(&genome_path)
        .output()
        .expect("bedtools shift failed");
    let bt_str = String::from_utf8(bt.stdout).unwrap();

    let mut ours_lines: Vec<&str> = ours_str.lines().filter(|l| !l.is_empty()).collect();
    let mut bt_lines: Vec<&str> = bt_str.lines().filter(|l| !l.is_empty()).collect();
    ours_lines.sort_unstable();
    bt_lines.sort_unstable();

    assert_eq!(
        ours_lines, bt_lines,
        "output differs from bedtools shift -s 100 -g genome.txt"
    );
}
