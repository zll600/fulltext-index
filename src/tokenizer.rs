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
        let mut char_offset = 0;
        
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
    fn test_tokenizer() {
        let tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize("The quick brown fox jumps over the lazy dog!");
        
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].text, "quick");
        assert_eq!(tokens[1].text, "brown");
        assert_eq!(tokens[2].text, "fox");
        assert_eq!(tokens[3].text, "jumps");
        assert_eq!(tokens[4].text, "over");
        assert_eq!(tokens[5].text, "lazy");
    }

    #[test]
    fn test_stemmer() {
        assert_eq!(SimpleStemmer::stem("running"), "runn");
        assert_eq!(SimpleStemmer::stem("jumped"), "jump");
        assert_eq!(SimpleStemmer::stem("quickly"), "quick");
        assert_eq!(SimpleStemmer::stem("boxes"), "box");
        assert_eq!(SimpleStemmer::stem("cats"), "cat");
        assert_eq!(SimpleStemmer::stem("class"), "class");
    }
}