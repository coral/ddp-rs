//! Send / frame-building hot-path benchmarks (the `no_std` core, no syscalls).
//!
//! Measures the allocation-free outgoing path that runs identically on host and on
//! bare-metal targets:
//!   - `header_serialize`: [`Header::write_into`] (write directly into a scratch buffer) vs
//!     the `Into<[u8; 10]>` conversion (build an array, then the caller copies it).
//!   - `build_single`: one [`FrameBuilder::for_each_frame`] call per payload size.
//!   - `build_large_multiframe`: a payload that chunks into several MTU-sized frames, to
//!     expose the per-chunk header-write + payload-copy cost.
//!   - `build_10k`: 10,000 full-MTU frames back to back — the headline send throughput.
//!
//! No socket I/O here; see `benches/connection.rs` for the framing + syscall cost.
//!
//! Run with: `cargo bench --bench send`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ddp_rs::protocol::{FrameBuilder, Header, PixelConfig, ID};

/// `num_pixels * 3` bytes of RGB payload.
fn make_data(num_pixels: usize) -> Vec<u8> {
    (0..num_pixels * 3).map(|i| (i % 256) as u8).collect()
}

fn bench_header_serialize(c: &mut Criterion) {
    let header = Header::default();
    let mut buf = [0u8; 14];

    let mut group = c.benchmark_group("header_serialize");
    group.throughput(Throughput::Elements(1));
    group.bench_function("write_into", |b| {
        b.iter(|| black_box(black_box(&header).write_into(black_box(&mut buf))))
    });
    group.bench_function("Into<[u8;10]>", |b| {
        b.iter(|| {
            let arr: [u8; 10] = black_box(header).into();
            black_box(arr)
        })
    });
    group.finish();
}

fn bench_build_single(c: &mut Criterion) {
    let mut scratch = [0u8; 1500];
    let mut group = c.benchmark_group("build_single");
    for &n in &[1usize, 60, 480] {
        let data = make_data(n);
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::new("for_each_frame", n), &data, |b, data| {
            let mut fb = FrameBuilder::new(PixelConfig::default(), ID::Default);
            b.iter(|| {
                fb.for_each_frame(black_box(data.as_slice()), 0, &mut scratch, |frame| {
                    black_box(frame);
                    Ok::<(), ()>(())
                })
                .unwrap()
            })
        });
    }
    group.finish();
}

fn bench_build_large_multiframe(c: &mut Criterion) {
    // 4096 pixels = 12288 bytes -> ceil(12288 / 1440) = 9 frames.
    let data = make_data(4096);
    let mut scratch = [0u8; 1500];

    let mut group = c.benchmark_group("build_large_multiframe");
    group.throughput(Throughput::Bytes(data.len() as u64));
    group.bench_function("for_each_frame", |b| {
        let mut fb = FrameBuilder::new(PixelConfig::default(), ID::Default);
        b.iter(|| {
            let mut acc = 0u64;
            fb.for_each_frame(black_box(data.as_slice()), 0, &mut scratch, |frame| {
                acc += frame.len() as u64;
                Ok::<(), ()>(())
            })
            .unwrap();
            black_box(acc)
        })
    });
    group.finish();
}

fn bench_build_10k(c: &mut Criterion) {
    const N: usize = 10_000;
    let data = make_data(480); // one full-MTU frame per call
    let mut scratch = [0u8; 1500];

    let mut group = c.benchmark_group("build_10k");
    group.throughput(Throughput::Bytes(N as u64 * data.len() as u64));
    group.sample_size(20);
    group.bench_function("for_each_frame x10k", |b| {
        let mut fb = FrameBuilder::new(PixelConfig::default(), ID::Default);
        b.iter(|| {
            let mut acc = 0u64;
            for _ in 0..N {
                fb.for_each_frame(black_box(data.as_slice()), 0, &mut scratch, |frame| {
                    acc += frame.len() as u64;
                    Ok::<(), ()>(())
                })
                .unwrap();
            }
            black_box(acc)
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_header_serialize,
    bench_build_single,
    bench_build_large_multiframe,
    bench_build_10k
);
criterion_main!(benches);
