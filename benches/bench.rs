#![feature(test)]
extern crate test;
use typo_checker::{TypoSuggestion, VecTypoChecker, TrieTypoChecker};

#[bench]
#[ignore]
fn bench_vec_read_dictionary(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        VecTypoChecker::new();
    });
}

#[bench]
#[ignore]
fn bench_trie_read_dictionary(bencher: &mut test::Bencher) {
    bencher.iter(|| {
        TrieTypoChecker::new();
    });
}

#[bench]
fn bench_vec_search(bencher: &mut test::Bencher) {
    let typo_checker = VecTypoChecker::new();
    bencher.iter(|| {
        assert_eq!(typo_checker.is_typo("doo"), true);
        assert_eq!(typo_checker.is_typo("lettuce"), false);
    });
}

#[bench]
fn bench_trie_search(bencher: &mut test::Bencher) {
    let typo_checker = TrieTypoChecker::new();
    bencher.iter(|| {
        assert_eq!(typo_checker.is_typo("doo"), true);
        assert_eq!(typo_checker.is_typo("lettuce"), false);
    });
}
