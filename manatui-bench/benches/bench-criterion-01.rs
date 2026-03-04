use criterion::{Criterion, criterion_group, criterion_main};

use manatui_bench::{basic_render, complex_render};

pub fn criterion_layout(c: &mut Criterion) {
    c.bench_function("basic_render", |b| b.iter(basic_render));
    c.bench_function("complex_render", |b| b.iter(complex_render));
}

criterion_group!(criterion_benches, criterion_layout);
criterion_main!(criterion_benches);
