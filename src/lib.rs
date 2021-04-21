#![feature(rustc_private)]
extern crate rustc_span;

pub trait TypoSuggestion {
    /** OS_DICTIONARY_PATH
    macos/raspbian: os built-in diction
    ubuntu: sudo apt install wbritish
    archlinux: sudo pacman -S words
    */
    const OS_DICTIONARY_PATH: &'static str = "/usr/share/dict/words";
    const MAX_EDIT_DISTANCE: usize = 1;
    const NUMBER_OF_SUGGESTIONS: usize = 5;
    fn new() -> Self;
    fn is_typo(&self, input_word: &str) -> bool;
    fn typo_suggestions(&self, input_word: &str) -> Vec<String>;
}

pub struct VecStringTypoChecker {
    words: Vec<String>,
}

impl TypoSuggestion for VecStringTypoChecker {
    fn new() -> Self {
        use std::io::{BufRead, BufReader};
        let mut words = vec![];
        let word_file = BufReader::new(std::fs::File::open(Self::OS_DICTIONARY_PATH).unwrap());
        for word in word_file.lines().flatten() {
            words.push(word);
        }
        Self { words }
    }

    fn is_typo(&self, input_word: &str) -> bool {
        dbg!(self.words.len());
        !self.words.contains(&input_word.to_string())
    }

    fn typo_suggestions(&self, input_word: &str) -> Vec<String> {
        if !self.is_typo(&input_word.to_string()) {
            return vec![];
        }
        let mut suggestions = vec![];
        for word in self.words.iter() {
            let edit_distance = rustc_span::lev_distance::lev_distance(input_word, word);
            if edit_distance <= Self::MAX_EDIT_DISTANCE {
                suggestions.push(word.clone());
            }
            if suggestions.len() > Self::NUMBER_OF_SUGGESTIONS {
                break;
            }
        }
        suggestions
    }
}

#[test]
fn test_vec_string_typo_checker() {
    let _ = VecStringTypoChecker::new();
}

#[test]
fn test_typo_checker() {
    let typo_checker = VecStringTypoChecker::new();
    let input_word = "doo";
    println!(
        "Unknown word `{}`, did you mean one of {:?}?",
        input_word,
        typo_checker.typo_suggestions(input_word)
    );
}

/*
fn download_english_words() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let mut http_response = Vec::new();
    let mut easy = curl::easy::Easy::new();
    // english words corpus: github.com/dwyl/english-words
    easy.url(
        "https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt",
    )?;
    let mut transfer = easy.transfer();
    transfer.write_function(|data| {
        http_response.extend_from_slice(data);
        Ok(data.len())
    })?;
    transfer.perform()?;
    drop(transfer);
    // cache words list to file
    let mut file = std::fs::File::create("english_words.txt")?;
    file.write_all(&http_response)?;
    Ok(())
}
*/
