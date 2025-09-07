use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub position: usize,
    pub start_offset: usize,
    pub end_offset: usize,
}

pub struct Tokenizer {
    stop_words: HashSet<String>,
    min_token_length: usize,
    max_token_length: usize,
}

impl Tokenizer {
    pub fn new() -> Self {
        let mut stop_words = HashSet::new();
        let common_stop_words = vec![
            "a", "an", "and", "are", "as", "at", "be", "been", "by", "for",
            "from", "has", "he", "in", "is", "it", "its", "of", "on", "that",
            "the", "to", "was", "will", "with", "the", "this", "these", "those",
            "i", "you", "we", "they", "them", "their", "what", "which", "who",
            "when", "where", "why", "how", "all", "would", "there", "been"
        ];
        
        for word in common_stop_words {
            stop_words.insert(word.to_string());
        }
        
        Self {
            stop_words,
            min_token_length: 2,
            max_token_length: 50,
        }
    }

    pub fn tokenize(&self, text: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut position = 0;
        
        let text_chars: Vec<char> = text.chars().collect();
        let mut current_word = String::new();
        let mut word_start = 0;
        
        for (i, ch) in text_chars.iter().enumerate() {
            if ch.is_alphanumeric() {
                if current_word.is_empty() {
                    word_start = i;
                }
                current_word.push(*ch);
            } else {
                if !current_word.is_empty() {
                    if let Some(token) = self.create_token(current_word.clone(), position, word_start, i) {
                        tokens.push(token);
                        position += 1;
                    }
                    current_word.clear();
                }
            }
        }
        
        if !current_word.is_empty() {
            if let Some(token) = self.create_token(current_word, position, word_start, text_chars.len()) {
                tokens.push(token);
            }
        }
        
        tokens
    }

    fn create_token(&self, text: String, position: usize, start: usize, end: usize) -> Option<Token> {
        let normalized = text.to_lowercase();
        
        if normalized.len() < self.min_token_length || normalized.len() > self.max_token_length {
            return None;
        }
        
        if self.stop_words.contains(&normalized) {
            return None;
        }
        
        Some(Token {
            text: normalized,
            position,
            start_offset: start,
            end_offset: end,
        })
    }

    pub fn add_stop_word(&mut self, word: &str) {
        self.stop_words.insert(word.to_lowercase());
    }

    pub fn remove_stop_word(&mut self, word: &str) {
        self.stop_words.remove(&word.to_lowercase());
    }

    pub fn set_min_token_length(&mut self, length: usize) {
        self.min_token_length = length;
    }

    pub fn set_max_token_length(&mut self, length: usize) {
        self.max_token_length = length;
    }
}

pub struct SimpleNormalizer;

impl SimpleNormalizer {
    pub fn normalize(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c.is_whitespace() {
                    c
                } else {
                    ' '
                }
            })
            .collect()
    }
}

pub struct SimpleStemmer;

impl SimpleStemmer {
    pub fn stem(word: &str) -> String {
        let word = word.to_lowercase();
        
        if word.ends_with("ing") && word.len() > 5 {
            word[..word.len() - 3].to_string()
        } else if word.ends_with("ed") && word.len() > 4 {
            word[..word.len() - 2].to_string()
        } else if word.ends_with("ly") && word.len() > 4 {
            word[..word.len() - 2].to_string()
        } else if word.ends_with("es") && word.len() > 4 {
            word[..word.len() - 2].to_string()
        } else if word.ends_with("s") && word.len() > 3 && !word.ends_with("ss") {
            word[..word.len() - 1].to_string()
        } else {
            word
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_basic() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("The quick brown fox jumps over the lazy dog!");
        
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].text, "quick");
        assert_eq!(tokens[1].text, "brown");
        assert_eq!(tokens[2].text, "fox");
        assert_eq!(tokens[3].text, "jumps");
        assert_eq!(tokens[4].text, "over");
        assert_eq!(tokens[5].text, "lazy");
    }

    #[test]
    fn test_tokenizer_positions() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("hello world test");
        
        assert_eq!(tokens.len(), 3);
        
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[0].position, 0);
        assert_eq!(tokens[0].start_offset, 0);
        assert_eq!(tokens[0].end_offset, 5);
        
        assert_eq!(tokens[1].text, "world");
        assert_eq!(tokens[1].position, 1);
        assert_eq!(tokens[1].start_offset, 6);
        assert_eq!(tokens[1].end_offset, 11);
        
        assert_eq!(tokens[2].text, "test");
        assert_eq!(tokens[2].position, 2);
        assert_eq!(tokens[2].start_offset, 12);
        assert_eq!(tokens[2].end_offset, 16);
    }

    #[test]
    fn test_tokenizer_stop_words() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("the quick brown fox is an animal");
        
        let token_texts: Vec<String> = tokens.iter().map(|t| t.text.clone()).collect();
        assert_eq!(token_texts, vec!["quick", "brown", "fox", "animal"]);
        
        assert!(!token_texts.contains(&"the".to_string()));
        assert!(!token_texts.contains(&"is".to_string()));
        assert!(!token_texts.contains(&"an".to_string()));
    }

    #[test]
    fn test_tokenizer_min_length() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.set_min_token_length(3);
        
        let tokens = tokenizer.tokenize("a bb ccc dddd");
        
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "ccc");
        assert_eq!(tokens[1].text, "dddd");
    }

    #[test]
    fn test_tokenizer_max_length() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.set_max_token_length(5);
        
        let tokens = tokenizer.tokenize("short verylongword medium");
        
        let token_texts: Vec<String> = tokens.iter().map(|t| t.text.clone()).collect();
        assert_eq!(token_texts, vec!["short"]);
        assert!(!token_texts.contains(&"verylongword".to_string()));
        assert!(!token_texts.contains(&"medium".to_string()));
    }

    #[test]
    fn test_tokenizer_punctuation() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("hello, world! computer programming test?");
        
        let token_texts: Vec<String> = tokens.iter().map(|t| t.text.clone()).collect();
        
        assert!(token_texts.contains(&"hello".to_string()));
        assert!(token_texts.contains(&"world".to_string()));
        assert!(token_texts.contains(&"computer".to_string()));
        assert!(token_texts.contains(&"programming".to_string()));
        assert!(token_texts.contains(&"test".to_string()));
        
        assert!(!token_texts.contains(&",".to_string()));
        assert!(!token_texts.contains(&"!".to_string()));
        assert!(!token_texts.contains(&"?".to_string()));
        
        assert_eq!(token_texts, vec!["hello", "world", "computer", "programming", "test"]);
    }

    #[test]
    fn test_tokenizer_numbers_and_alphanumeric() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("test123 456 abc def789");
        
        let token_texts: Vec<String> = tokens.iter().map(|t| t.text.clone()).collect();
        assert_eq!(token_texts, vec!["test123", "456", "abc", "def789"]);
    }

    #[test]
    fn test_tokenizer_empty_input() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("");
        
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_tokenizer_whitespace_only() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("   \t\n\r  ");
        
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_tokenizer_case_normalization() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("Hello WORLD Test");
        
        assert_eq!(tokens[0].text, "hello");
        assert_eq!(tokens[1].text, "world");
        assert_eq!(tokens[2].text, "test");
    }

    #[test]
    fn test_tokenizer_custom_stop_words() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.add_stop_word("custom");
        tokenizer.add_stop_word("special");
        
        let tokens = tokenizer.tokenize("this custom word and special term");
        
        let token_texts: Vec<String> = tokens.iter().map(|t| t.text.clone()).collect();
        assert_eq!(token_texts, vec!["word", "term"]);
    }

    #[test]
    fn test_tokenizer_remove_stop_word() {
        let mut tokenizer = Tokenizer::new();
        tokenizer.remove_stop_word("the");
        
        let tokens = tokenizer.tokenize("the quick brown fox");
        
        let token_texts: Vec<String> = tokens.iter().map(|t| t.text.clone()).collect();
        assert_eq!(token_texts, vec!["the", "quick", "brown", "fox"]);
    }

    #[test]
    fn test_tokenizer_unicode_characters() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("café naïve résumé");
        
        let token_texts: Vec<String> = tokens.iter().map(|t| t.text.clone()).collect();
        assert_eq!(token_texts, vec!["café", "naïve", "résumé"]);
    }

    #[test]
    fn test_normalizer() {
        let normalized = SimpleNormalizer::normalize("Hello, World! 123 @#$%");
        
        assert_eq!(normalized, "hello  world  123     ");
    }

    #[test]
    fn test_normalizer_preserve_whitespace() {
        let normalized = SimpleNormalizer::normalize("hello   world");
        
        assert_eq!(normalized, "hello   world");
    }

    #[test]
    fn test_stemmer_ing_suffix() {
        assert_eq!(SimpleStemmer::stem("running"), "runn");
        assert_eq!(SimpleStemmer::stem("walking"), "walk");
        assert_eq!(SimpleStemmer::stem("jumping"), "jump");
        
        assert_eq!(SimpleStemmer::stem("sing"), "sing");  // Length 4, would become 1 char
        assert_eq!(SimpleStemmer::stem("bring"), "bring"); // Length 5, would become 2 chars (minimum)
    }

    #[test]
    fn test_stemmer_ed_suffix() {
        assert_eq!(SimpleStemmer::stem("jumped"), "jump");
        assert_eq!(SimpleStemmer::stem("walked"), "walk");
        assert_eq!(SimpleStemmer::stem("tested"), "test");
        
        assert_eq!(SimpleStemmer::stem("used"), "used"); // Length 4, would become 2 chars
    }

    #[test]
    fn test_stemmer_ly_suffix() {
        assert_eq!(SimpleStemmer::stem("quickly"), "quick");
        assert_eq!(SimpleStemmer::stem("slowly"), "slow");
        assert_eq!(SimpleStemmer::stem("really"), "real");
        
        assert_eq!(SimpleStemmer::stem("only"), "only"); // Length 4, would become 2 chars
    }

    #[test]
    fn test_stemmer_es_suffix() {
        assert_eq!(SimpleStemmer::stem("boxes"), "box");
        assert_eq!(SimpleStemmer::stem("wishes"), "wish");
        assert_eq!(SimpleStemmer::stem("classes"), "class");
        
        assert_eq!(SimpleStemmer::stem("ges"), "ges"); // Length 3, too short
    }

    #[test]
    fn test_stemmer_s_suffix() {
        assert_eq!(SimpleStemmer::stem("cats"), "cat");
        assert_eq!(SimpleStemmer::stem("dogs"), "dog");
        assert_eq!(SimpleStemmer::stem("books"), "book");
        
        assert_eq!(SimpleStemmer::stem("class"), "class");
        assert_eq!(SimpleStemmer::stem("grass"), "grass");
        assert_eq!(SimpleStemmer::stem("pass"), "pass");
        
        assert_eq!(SimpleStemmer::stem("yes"), "yes"); // Length 3, would become 2 chars
    }

    #[test]
    fn test_stemmer_case_insensitive() {
        assert_eq!(SimpleStemmer::stem("Running"), "runn");
        assert_eq!(SimpleStemmer::stem("JUMPED"), "jump");
        assert_eq!(SimpleStemmer::stem("QuickLY"), "quick");
    }

    #[test]
    fn test_stemmer_no_suffix() {
        assert_eq!(SimpleStemmer::stem("computer"), "computer");
        assert_eq!(SimpleStemmer::stem("algorithm"), "algorithm");
        assert_eq!(SimpleStemmer::stem("data"), "data");
    }

    #[test]
    fn test_stemmer_empty_and_short_words() {
        // Empty string
        assert_eq!(SimpleStemmer::stem(""), "");
        
        // Very short words
        assert_eq!(SimpleStemmer::stem("a"), "a");
        assert_eq!(SimpleStemmer::stem("is"), "is");
        assert_eq!(SimpleStemmer::stem("the"), "the");
    }
}