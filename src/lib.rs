struct TypoChecker {
    /// TODO use tire data structure to store words for better performance
    words: Vec<String>
}

impl TypoChecker {
    fn new() -> Self {
        use std::io::{Write, BufRead, BufReader};
        const WORDS_FILENAME: &str = "english_words.txt";

        fn download_words_list() {
            let mut http_response = Vec::new();
            let mut easy = curl::easy::Easy::new();
            // english words corpus: github.com/dwyl/english-words
            easy.url("https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt").unwrap();
            let mut transfer = easy.transfer();
            transfer.write_function(|data| {
                http_response.extend_from_slice(data);
                Ok(data.len())
            }).unwrap();
            transfer.perform().unwrap();
            drop(transfer);
            // cache words list to file
            let mut file = std::fs::File::create(WORDS_FILENAME).unwrap();
            file.write_all(&http_response).unwrap();
        }
    
        if !std::path::Path::new(WORDS_FILENAME).exists() {
            download_words_list();
        }
        let mut words = vec![];
        let word_file = BufReader::new(std::fs::File::open(WORDS_FILENAME).unwrap());
        for line in word_file.lines().flatten() {
            words.push(line);
        }
        Self {
            words
        }
    }

    fn typo_suggestions(&self, input: &str) -> Vec<String> {
        let mut suggestions = vec![];
        for word in self.words.iter() {
            let edit_distance = rustc_span::lev_distance::lev_distance(input, word);
            if edit_distance < 2 {
                suggestions.push(word.clone());
            }
            if suggestions.len() > 5 {
                break;
            }
        }
        suggestions
    }
    // fn is_bad_word(&self, word: &String) -> bool { self.words.contains(word)}
}

#[test]
fn test_typo_checker() {
    let typo_checker = TypoChecker::new();
    let input = "doo";
    println!("Unknown word `{}`, did you mean one of {:?}?", input, typo_checker.typo_suggestions(input));
}
