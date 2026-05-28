use std::io::Write;
use std::process::Command;

use criterion::{Criterion, criterion_group, criterion_main};

fn make_fixture(n: usize) -> tempfile::NamedTempFile {
    use std::io::BufWriter;
    let mut f = tempfile::NamedTempFile::new().unwrap();
    {
        let mut w = BufWriter::new(&mut f);
        let chroms = ["chr1", "chr2", "chr3", "chr4"];
        for i in 0..n {
            let chrom = chroms[i % chroms.len()];
            let start = (i * 1000) % 100_000_000;
            let end = start + 500;
            let strand = if i % 2 == 0 { "+" } else { "-" };
            writeln!(w, "{chrom}\t{start}\t{end}\tfeat{i}\t0\t{strand}").unwrap();
        }
    } // w dropped here, releasing borrow on f
    f
}

fn make_genome() -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    for chrom in ["chr1", "chr2", "chr3", "chr4"] {
        writeln!(f, "{chrom}\t200000000").unwrap();
    }
    f
}

fn bench_shift(c: &mut Criterion) {
    let input = make_fixture(100_000);
    let genome = make_genome();

    let mut group = c.benchmark_group("bed-shift 100k records");

    group.bench_function("rsomics-bed-shift", |b| {
        b.iter(|| {
            let out = tempfile::NamedTempFile::new().unwrap();
            let status = Command::new(env!("CARGO_BIN_EXE_rsomics-bed-shift"))
                .arg("-s")
                .arg("100")
                .arg("-g")
                .arg(genome.path())
                .arg(input.path())
                .arg("--out")
                .arg(out.path())
                .status()
                .unwrap();
            assert!(status.success());
        })
    });

    if Command::new("bedtools").arg("--version").output().is_ok() {
        group.bench_function("bedtools-shift", |b| {
            b.iter(|| {
                let out = tempfile::NamedTempFile::new().unwrap();
                let status = Command::new("bedtools")
                    .args(["shift", "-i"])
                    .arg(input.path())
                    .arg("-s")
                    .arg("100")
                    .arg("-g")
                    .arg(genome.path())
                    .stdout(std::fs::File::create(out.path()).unwrap())
                    .status()
                    .unwrap();
                assert!(status.success());
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_shift);
criterion_main!(benches);
