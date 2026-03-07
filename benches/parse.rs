use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ion::{Ion, Parser};
use std::str::FromStr;

const DEF_HOTEL_ON_START: &str = include_str!("data/def_hotel_on_start.ion");
const DEF_HOTEL_ON_END: &str = include_str!("data/def_hotel_on_end.ion");
const FILTERED_SECTIONS: &[&str] = &["CONTRACT", "DEF.HOTEL"];

fn parse_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    group.bench_function("section_on_start_of_ion", |b| {
        b.iter(|| {
            let result = Ion::from_str(DEF_HOTEL_ON_START);
            black_box(result.unwrap())
        })
    });

    group.bench_function("section_on_end_of_ion", |b| {
        b.iter(|| {
            let result = Ion::from_str(DEF_HOTEL_ON_END);
            black_box(result.unwrap())
        })
    });

    group.bench_function("section_on_start_of_ion_tuned_parser", |b| {
        b.iter(|| {
            let result = Parser::new(DEF_HOTEL_ON_START)
                .with_row_capacity(12)
                .with_array_capacity(4)
                .with_section_capacity(1024)
                .read();

            black_box(result.unwrap())
        })
    });

    group.bench_function("section_on_start_of_ion_parser_no_prealloc", |b| {
        b.iter(|| {
            let result = Parser::new(DEF_HOTEL_ON_START)
                .with_row_capacity(0)
                .with_array_capacity(0)
                .with_section_capacity(0)
                .read();

            black_box(result.unwrap())
        })
    });

    group.bench_function("section_on_end_of_ion_tuned_parser", |b| {
        b.iter(|| {
            let result = Parser::new(DEF_HOTEL_ON_END)
                .with_row_capacity(12)
                .with_array_capacity(4)
                .with_section_capacity(1024)
                .read();

            black_box(result.unwrap())
        })
    });

    group.bench_function("section_on_end_of_ion_parser_no_prealloc", |b| {
        b.iter(|| {
            let result = Parser::new(DEF_HOTEL_ON_END)
                .with_row_capacity(0)
                .with_array_capacity(0)
                .with_section_capacity(0)
                .read();

            black_box(result.unwrap())
        })
    });

    group.finish();
}

fn parse_filtered_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_filtered");

    group.bench_function("section_on_start_of_ion", |b| {
        b.iter(|| {
            let result = Ion::from_str_filtered(DEF_HOTEL_ON_START, FILTERED_SECTIONS.to_vec());
            black_box(result.unwrap())
        })
    });

    group.bench_function("section_on_end_of_ion", |b| {
        b.iter(|| {
            let result = Ion::from_str_filtered(DEF_HOTEL_ON_END, FILTERED_SECTIONS.to_vec());
            black_box(result.unwrap())
        })
    });

    group.finish();
}

criterion_group!(benches, parse_benches, parse_filtered_benches);
criterion_main!(benches);
