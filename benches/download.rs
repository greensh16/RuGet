use criterion::{criterion_group, criterion_main, Criterion};
use std::process::Command;

fn bench_download(c: &mut Criterion) {
    c.bench_function("download example.com", |b| {
        b.iter(|| {
            let _ = Command::new("./target/debug/rustget")
                .args(["https://example.com", "--output", "bench_output.html"])
                .output()
                .unwrap();
        });
    });
}

criterion_group!(benches, bench_download);
criterion_main!(benches);