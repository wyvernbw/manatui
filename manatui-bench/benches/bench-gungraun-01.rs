#![expect(clippy::unit_arg)]

use std::hint::black_box;

use gungraun::{library_benchmark, library_benchmark_group, main};
use manatui_bench::basic_render;

#[library_benchmark]
fn bench_basic_render() {
    black_box(basic_render());
}

library_benchmark_group!(name = basic_render, benchmarks = bench_basic_render);
main!(library_benchmark_groups = basic_render);
