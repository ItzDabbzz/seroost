#[derive(Debug)]
pub struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    pub const fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) {
        while !self.content.is_empty() && self.content[0].is_whitespace() {
            self.content = &self.content[1..];
        }
    }

    fn chop_while<P>(&mut self, mut predicate: P) -> &'a [char]
    where
        P: FnMut(&char) -> bool,
    {
        let mut n = 0;
        // This token begins at an alphabet and ends at non-alphanumeric characters like (*, ^, &,. etc)
        while n < self.content.len() && predicate(&self.content[n]) {
            n += 1; // Increment token width
        }
        self.chop(n) // Take the required amount.
    }
    fn chop(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[0..n]; // Get a slice of the current content.
        self.content = &self.content[n..]; // The new content excludes the previous token.

        token
    }
    fn next_token(&mut self) -> Option<String> {
        // trim whitespaces from left.
        self.trim_left();

        if self.content.is_empty() {
            return None;
        }
        if self.content[0].is_alphabetic() {
            Some(
                self.chop_while(|x| x.is_alphanumeric())
                    .iter()
                    .map(char::to_ascii_lowercase)
                    .collect::<String>(),
            )
        } else if self.content[0].is_numeric() {
            Some(
                self.chop_while(|x| x.is_numeric() || x.is_alphabetic())
                    .iter()
                    .collect::<String>(),
            )
        } else {
            Some(self.chop(1).iter().collect::<String>())
        }
    }
}

// Implement the Iteratort trait for the lexer.
impl Iterator for Lexer<'_> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_word() {
        let content: Vec<char> = "hello".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["hello"]);
    }

    #[test]
    fn test_multiple_words() {
        let content: Vec<char> = "hello world foo".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["hello", "world", "foo"]);
    }

    #[test]
    fn test_lowercase_conversion() {
        let content: Vec<char> = "Hello WORLD".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_numbers() {
        let content: Vec<char> = "abc123".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["abc123"]);
    }

    #[test]
    fn test_separate_numbers() {
        let content: Vec<char> = "123 456".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["123", "456"]);
    }

    #[test]
    fn test_special_chars() {
        let content: Vec<char> = "@#$".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["@", "#", "$"]);
    }

    #[test]
    fn test_mixed_content() {
        let content: Vec<char> = "Hello, World! 123".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["hello", ",", "world", "!", "123"]);
    }

    #[test]
    fn test_whitespace_handling() {
        let content: Vec<char> = "  hello   world  ".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_empty_input() {
        let content: Vec<char> = "".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let content: Vec<char> = "   \t\n  ".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_long_sentence() {
        let content: Vec<char> = "The quick brown fox jumps over the lazy dog"
            .chars()
            .collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(
            tokens,
            vec!["the", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"]
        );
    }

    #[test]
    fn test_single_char_word() {
        let content: Vec<char> = "a b c d".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_unicode_characters() {
        let content: Vec<char> = "café résumé naïve".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["café", "résumé", "naïve"]);
    }

    #[test]
    fn test_leading_whitespace_types() {
        let content: Vec<char> = "\t\n\r  hello".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["hello"]);
    }

    #[test]
    fn test_consecutive_special_chars_no_space() {
        let content: Vec<char> = "!!!@@@###".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["!", "!", "!", "@", "@", "@", "#", "#", "#"]);
    }

    #[test]
    fn test_hyphenated_word() {
        let content: Vec<char> = "hello-world foo-bar".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["hello", "-", "world", "foo", "-", "bar"]);
    }

    #[test]
    fn test_underscore_in_word() {
        let content: Vec<char> = "hello_world foo".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["hello", "_", "world", "foo"]);
    }

    #[test]
    fn test_number_leading_zeros() {
        let content: Vec<char> = "007 00100".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["007", "00100"]);
    }

    #[test]
    fn test_mixed_whitespace_between_tokens() {
        let content: Vec<char> = "a\t\tb\n\nc\rd".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_trailing_special_chars() {
        let content: Vec<char> = "hello!@#".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["hello", "!", "@", "#"]);
    }

    #[test]
    fn test_number_leading_alpha_token() {
        let content: Vec<char> = "123hello".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["123hello"]);
    }

    #[test]
    fn test_single_alpha_char() {
        let content: Vec<char> = "a".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["a"]);
    }

    #[test]
    fn test_single_number_char() {
        let content: Vec<char> = "5".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["5"]);
    }

    #[test]
    fn test_single_special_char() {
        let content: Vec<char> = "!".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["!"]);
    }

    #[test]
    fn test_capital_number_lowercase_mixed() {
        let content: Vec<char> = "AB123cd".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["ab123cd"]);
    }

    #[test]
    fn test_only_newlines_and_tabs() {
        let content: Vec<char> = "\n\n\t\t\n".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_multiple_spaces_between_special_chars() {
        let content: Vec<char> = "@  #  $".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["@", "#", "$"]);
    }

    #[test]
    fn test_empty_slice_directly() {
        let content: [char; 0] = [];
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_alternating_alpha_and_special() {
        let content: Vec<char> = "a-b-c-d".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["a", "-", "b", "-", "c", "-", "d"]);
    }

    #[test]
    fn test_punctuation_in_sentence() {
        let content: Vec<char> = "Hello, world. How are you?".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(
            tokens,
            vec!["hello", ",", "world", ".", "how", "are", "you", "?"]
        );
    }

    #[test]
    fn test_numbers_with_letters_mixed() {
        let content: Vec<char> = "test123abc456".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["test123abc456"]);
    }

    #[test]
    fn test_only_digits() {
        let content: Vec<char> = "9876543210".chars().collect();
        let lexer = Lexer::new(&content);
        let tokens: Vec<String> = lexer.collect();
        assert_eq!(tokens, vec!["9876543210"]);
    }
}
