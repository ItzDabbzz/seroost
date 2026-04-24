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
                self.chop_while(|x| x.is_numeric())
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
