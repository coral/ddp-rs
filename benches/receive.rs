//! Receive / parse hot-path benchmarks.
//!
//! Compares the two receive APIs over the same byte buffers:
//!   - [`PacketRef::from_bytes`] — the zero-copy `no_std` core path (what runs on Teensy /
//!     ESP32). Borrows the payload, no allocation.
//!   - [`Packet::from_bytes`] — the owned `alloc`/`std` path. Copies the payload into a fresh
//!     `Vec` (`data.to_vec()`) and, for reply packets, attempts JSON parsing.
//!
//! The `parse_10k` group is the headline: parsing 10,000 full-MTU packets back to back, which
//! is where the per-packet allocation in `Packet` shows up against the zero-copy `PacketRef`.
//!
//! Run with: `cargo bench --bench receive`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ddp_rs::packet::{Packet, PacketRef};
use ddp_rs::protocol::Header;

const HEADER_LEN: usize = 10;

/// Builds a valid RGB pixel packet (10-byte header, push flag set, `num_pixels * 3` payload).
fn make_pixel_packet(num_pixels: usize) -> Vec<u8> {
    let len = num_pixels * 3;
    let mut v = vec![0x41, 0x01, 0x0D, 0x01]; // v1+push, seq=1, RGB config, id=Default
    v.extend_from_slice(&0u32.to_be_bytes()); // offset
    v.extend_from_slice(&(len as u16).to_be_bytes()); // length
    v.extend((0..len).map(|i| (i % 256) as u8)); // payload
    v
}

/// Representative payload sizes: a single pixel, a 60-LED ring, and a full DDP MTU (480 px).
const PIXEL_COUNTS: &[usize] = &[1, 60, 480];

fn bench_header_parse(c: &mut Criterion) {
    let pkt = make_pixel_packet(480);
    let mut group = c.benchmark_group("header_parse");
    group.throughput(Throughput::Elements(1));
    group.bench_function("Header::from(10b)", |b| {
        b.iter(|| Header::from(black_box(&pkt[..HEADER_LEN])))
    });
    group.finish();
}

fn bench_single_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_single");
    for &n in PIXEL_COUNTS {
        let pkt = make_pixel_packet(n);
        group.throughput(Throughput::Bytes(pkt.len() as u64));
        group.bench_with_input(BenchmarkId::new("PacketRef", n), &pkt, |b, pkt| {
            b.iter(|| PacketRef::from_bytes(black_box(pkt.as_slice())).unwrap())
        });
        group.bench_with_input(BenchmarkId::new("Packet", n), &pkt, |b, pkt| {
            b.iter(|| Packet::from_bytes(black_box(pkt.as_slice())))
        });
    }
    group.finish();
}

fn bench_parse_10k(c: &mut Criterion) {
    const N: usize = 10_000;
    let pkts: Vec<Vec<u8>> = (0..N).map(|_| make_pixel_packet(480)).collect();
    let total_bytes: u64 = pkts.iter().map(|p| p.len() as u64).sum();

    let mut group = c.benchmark_group("parse_10k");
    group.throughput(Throughput::Bytes(total_bytes));
    group.sample_size(20);

    group.bench_function("PacketRef (zero-copy)", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            for p in &pkts {
                let pr = PacketRef::from_bytes(black_box(p.as_slice())).unwrap();
                // Touch the parsed result so the work can't be optimized away.
                acc += pr.data.len() as u64 + pr.header.offset as u64;
            }
            black_box(acc)
        })
    });

    group.bench_function("Packet (owned/alloc)", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            for p in &pkts {
                let pk = Packet::from_bytes(black_box(p.as_slice()));
                acc += pk.data.len() as u64 + pk.header.offset as u64;
            }
            black_box(acc)
        })
    });
    group.finish();
}

criterion_group!(benches, bench_header_parse, bench_single_parse, bench_parse_10k);
criterion_main!(benches);
