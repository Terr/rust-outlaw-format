use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use outlaw_format::{consts, format, format_to_string, parse_document, wrap_long_lines};

fn bench_format_outlaw_file(c: &mut Criterion) {
    let outlaw_file = include_str!("long_document.input");

    c.bench_function("format outlaw file", |b| {
        b.iter(|| format(black_box(outlaw_file)))
    });
}

fn bench_parse_document(c: &mut Criterion) {
    let outlaw_file = include_str!("long_document.input");

    c.bench_function("parse document", |b| {
        b.iter(|| parse_document(black_box(outlaw_file)))
    });
}

fn bench_wrap_long_lines(c: &mut Criterion) {
    c.bench_function("parse document + wrap long lines", |b| {
        b.iter(|| {
            let mut document = parse_document(include_str!("long_document.input"));
            for block in document.blocks.iter_mut() {
                wrap_long_lines(&mut block.contents, consts::MAX_LINE_LENGTH);
            }
        })
    });
}

fn bench_format_to_string(c: &mut Criterion) {
    let document = parse_document(include_str!("long_document.input"));

    c.bench_function("format to string", |b| {
        b.iter(|| format_to_string(black_box(&document)))
    });
}

criterion_group!(
    benches,
    bench_format_outlaw_file,
    bench_parse_document,
    bench_wrap_long_lines,
    bench_format_to_string,
);
criterion_main!(benches);
