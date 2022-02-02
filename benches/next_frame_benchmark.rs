#[macro_use]
extern crate criterion;

use criterion::{criterion_group, criterion_main, Criterion};

use pprof::criterion::{Output, PProfProfiler};

use cheval::render_buffer::RenderBuffer;
use cheval::window::window_minifb::WindowMinifb;

fn render_frame_rgb_a( source: &RenderBuffer, width: usize, height: usize, dest_rgb: &mut Vec< u32 >, dest_a: &mut Vec< u32 > ) {
    WindowMinifb::render_frame_rgb_a( source, width, height, dest_rgb, dest_a );
}

fn criterion_benchmark(c: &mut Criterion) {
    let width       = 1920;
    let height      = 1080;
    let downscale   = 2;

    let source  = RenderBuffer::new( width, height );
    let width   = width / downscale;
    let height  = height / downscale;

    let mut dest_rgb    = vec![0u32; width * height];
    let mut dest_a      = vec![0u32; width * height];

    c.bench_function("render RGB_A", |b| b.iter(|| render_frame_rgb_a( &source, width, height, &mut dest_rgb, &mut dest_a )));
}

//criterion_group!(benches, criterion_benchmark);
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}
criterion_main!(benches);
