use std::{
    fs::File,
    io::{Cursor, Read},
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use tinyvg::Decoder;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("TinyVG");

    let mut f = File::open("data/tiger.tvg").unwrap();
    let mut data = Vec::new();
    f.read_to_end(&mut data).unwrap();

    g.bench_function(BenchmarkId::new("decode", "tiger.tvg"), |b| {
        b.iter(|| {
            let p = Decoder::new(Cursor::new(&data));

            black_box(p.decode().unwrap());
        })
    });

    g.bench_function(BenchmarkId::new("render", "tiger.tvg"), |b| {
        let p = Decoder::new(Cursor::new(&data));
        let image = p.decode().unwrap();
        let mut data = Vec::new();

        b.iter(|| {
            black_box(image.render_png(&mut data).unwrap());
            data.clear();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
