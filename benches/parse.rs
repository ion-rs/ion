extern crate test;

use ion::{Ion, Parser};
use test::{Bencher, black_box};

const DEF_HOTEL_ON_START: &str = include_str!("data/def_hotel_on_start.ion");
const DEF_HOTEL_ON_END: &str = include_str!("data/def_hotel_on_end.ion");

mod parse {
    use super::*;
    use std::str::FromStr;

    #[bench]
    fn section_on_start_of_ion(bencher: &mut Bencher) {
        bencher.iter(|| {
            let result = Ion::from_str(DEF_HOTEL_ON_START);
            black_box(result.unwrap())
        })
    }

    #[bench]
    fn section_on_end_of_ion(bencher: &mut Bencher) {
        bencher.iter(|| {
            let result = Ion::from_str(DEF_HOTEL_ON_END);
            black_box(result.unwrap())
        })
    }

    #[bench]
    fn section_on_start_of_ion_tuned_parser(bencher: &mut Bencher) {
        bencher.iter(|| {
            let result = Parser::new(DEF_HOTEL_ON_START)
                .with_row_capacity(12)
                .with_array_capacity(4)
                .with_section_capacity(1024)
                .read();

            black_box(result.unwrap())
        })
    }

    #[bench]
    fn section_on_start_of_ion_parser_no_prealloc(bencher: &mut Bencher) {
        bencher.iter(|| {
            let result = Parser::new(DEF_HOTEL_ON_START)
                .with_row_capacity(0)
                .with_array_capacity(0)
                .with_section_capacity(0)
                .read();

            black_box(result.unwrap())
        })
    }

    #[bench]
    fn section_on_end_of_ion_tuned_parser(bencher: &mut Bencher) {
        bencher.iter(|| {
            let result = Parser::new(DEF_HOTEL_ON_END)
                .with_row_capacity(12)
                .with_array_capacity(4)
                .with_section_capacity(1024)
                .read();

            black_box(result.unwrap())
        })
    }

    #[bench]
    fn section_on_end_of_ion_parser_no_prealloc(bencher: &mut Bencher) {
        bencher.iter(|| {
            let result = Parser::new(DEF_HOTEL_ON_END)
                .with_row_capacity(0)
                .with_array_capacity(0)
                .with_section_capacity(0)
                .read();

            black_box(result.unwrap())
        })
    }
}

mod parse_filtered {
    use super::*;

    const FILTERED_SECTIONS: &[&str] = &["CONTRACT", "DEF.HOTEL"];

    #[bench]
    fn section_on_start_of_ion(bencher: &mut Bencher) {
        bencher.iter(|| {
            let result = Ion::from_str_filtered(DEF_HOTEL_ON_START, FILTERED_SECTIONS.to_vec());
            black_box(result.unwrap())
        })
    }

    #[bench]
    fn section_on_end_of_ion(bencher: &mut Bencher) {
        bencher.iter(|| {
            let result = Ion::from_str_filtered(DEF_HOTEL_ON_END, FILTERED_SECTIONS.to_vec());
            black_box(result.unwrap())
        })
    }
}
