#![feature(rustc_private)]
extern crate rustc_span;

pub trait TypoSuggestion: Sized + Default {
    const MAX_EDIT_DISTANCE: usize = 1;
    const NUMBER_OF_SUGGESTIONS: usize = 5;
    fn insert(&mut self, word: String);
    fn read_os_dictionary(&mut self) {
        /** OS_DICTIONARY_PATH
        macos/raspbian: os built-in diction
        ubuntu: sudo apt install wbritish
        archlinux: sudo pacman -S words
        or download https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt
        */
        const OS_DICTIONARY_PATH: &str = "/usr/share/dict/words";
        use std::io::{BufRead, BufReader};
        let word_file = BufReader::new(std::fs::File::open(OS_DICTIONARY_PATH).unwrap());
        for word in word_file.lines().flatten() {
            self.insert(word)
        }
    }
    /// return type Self must bound Sized
    fn new() -> Self {
        let mut typo_checker = Self::default();
        typo_checker.read_os_dictionary();
        typo_checker
    }
    fn is_typo(&self, word: &str) -> bool;
    fn typo_suggestions(&self, word: &str) -> Vec<String>;
}

#[derive(Default)]
pub struct VecTypoChecker {
    words: Vec<String>,
}

impl TypoSuggestion for VecTypoChecker {
    fn insert(&mut self, word: String) {
        self.words.push(word);
    }

    fn is_typo(&self, word: &str) -> bool {
        !self.words.contains(&word.to_string())
    }

    fn typo_suggestions(&self, word: &str) -> Vec<String> {
        let input_word = word.to_string();
        if !self.is_typo(&input_word) {
            return vec![];
        }
        let mut suggestions = vec![];
        for word in self.words.iter() {
            let edit_distance = rustc_span::lev_distance::lev_distance(&input_word, word);
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

#[derive(Default)]
pub struct TrieTypoChecker {
    children: [Option<Box<Self>>; 26],
    is_word: bool,
}

impl TypoSuggestion for TrieTypoChecker {
    fn insert(&mut self, word: String) {
        let word = word
            .into_bytes()
            .into_iter()
            .map(|ch| ch.to_ascii_lowercase())
            .filter(|ch| matches!(ch, b'a'..=b'z'))
            .collect::<Vec<u8>>();
        let mut curr_node = self;
        for letter in word.into_iter().map(|ch| (ch - b'a') as usize) {
            curr_node = curr_node.children[letter].get_or_insert_with(|| Box::new(Self::default()))
        }
        curr_node.is_word = true;
    }

    fn is_typo(&self, word: &str) -> bool {
        let word = word.as_bytes();
        let mut curr_node = self;
        for letter in word {
            let index = (letter - b'a') as usize;
            match curr_node.children[index] {
                Some(ref child_node) => {
                    curr_node = child_node.as_ref();
                }
                None => {
                    return true;
                }
            }
        }
        !curr_node.is_word
    }

    fn typo_suggestions(&self, word: &str) -> Vec<String> {
        let mut dfs_helper = DfsHelper {
            input_word: word.as_bytes().to_vec(),
            input_word_len: word.len(),
            output_suggestions: vec![],
            path: vec![],
        };
        dfs_helper.dfs(&self, 0, 1);
        dfs_helper.output_suggestions
    }
}

/// [鹅厂面试题，英语单词拼写检查算法？](https://www.zhihu.com/question/29592463)
/// 为了偷懒，把dfs一些不需要回溯的递归间全局共享的状态放到一个结构体
struct DfsHelper {
    /// 输入的单词
    input_word: Vec<u8>,
    input_word_len: usize,
    /// 返回值
    output_suggestions: Vec<String>,
    /// 当前深度优先搜索，从根节点到当前节点的路径(path root to curr_node)
    path: Vec<u8>,
}

impl DfsHelper {
    fn dfs(&mut self, curr_node: &TrieTypoChecker, input_word_index: usize, edit_times: i32) {
        if edit_times < 0 {
            return;
        }

        if input_word_index == self.input_word_len {
            if curr_node.is_word {
                self.output_suggestions.push(unsafe { String::from_utf8_unchecked(self.path.clone()) });
            }
            if edit_times == 0 {
                return;
            }
            // 输入单词遍历遍历完了，如果还有编辑次数可用，则用剩余的编辑次数给当前dfs遍历路径组成的单词词尾巴追加字母
            // 例如 input_word="do", trie从根到当前节点的路径d->o遍历完还剩余1次编辑次数，则可以用做增加操作，把g加到当前路径中
            for (i, child_node_opt) in curr_node.children.iter().take(26).enumerate() {
                if let Some(child_node) = child_node_opt {
                    self.path.push(b'a' + i as u8);
                    self.dfs(child_node, input_word_index, edit_times-1);
                    self.path.pop().unwrap();
                }
            }
            return;
        }


        if self.output_suggestions.len() >= TrieTypoChecker::NUMBER_OF_SUGGESTIONS {
            return;
        }

        let curr_letter_index = (self.input_word[input_word_index] - b'a') as usize;
        for (i, child_node_opt) in curr_node.children.iter().take(26).enumerate() {
            if let Some(child_node) = child_node_opt {
                if i == curr_letter_index {
                    self.path.push(self.input_word[input_word_index]);
                    self.dfs(child_node, input_word_index+1, edit_times);
                    self.path.pop();
                } else {
                    // replace
                    self.path.push(b'a' + i as u8);
                    self.dfs(child_node, input_word_index+1, edit_times-1);
                    self.path.pop().unwrap();
                }
            }
        }

    }
}

/*
thread 'test_trie_typo_checker' panicked at 'assertion failed: `(left == right)`
  left: `["boo", "coo", "doa", "dob", "doc", "dod", "doe", "dog", "don", "doom", "door", "dos", "dot", "dow", "doz"]`,
 right: `["boo", "coo", "dao", "do", "doa", "dob"]`', src/lib.rs:182:9

 TODO support delete operation
*/
#[test]
fn test_trie_typo_checker() {
    const TEST_CASES: [(&str, &[&str]); 1] = [
        ("doo", &["boo", "coo", "dao", "do", "doa", "dob"])
    ];
    let typo_checker = TrieTypoChecker::new();
    for (input, output) in std::array::IntoIter::new(TEST_CASES) {
        assert_eq!(typo_checker.typo_suggestions(input), output);
    }
}

#[test]
fn test_typo_checker() {
    let typo_checker = TrieTypoChecker::new();
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
