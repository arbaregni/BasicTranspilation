use std::vec::Vec;

#[derive(Copy, Clone, Debug)]
pub struct Token {
    begin: usize,
    len: usize,
}

impl Token {
    pub fn new(begin: usize, len: usize) -> Token {
        Token{begin, len}
    }
    pub fn get_text<'a, 'b>(&'a self, source: &'b str) -> &'b str {
        &source[self.begin..self.begin+self.len]
    }
    pub fn get_underlined(&self, source: &str) -> String {
        let mut line_start = self.begin;
        while line_start > 0 && &source[line_start - 1..line_start] != "\n" {
            line_start -= 1;
        }
        let mut line_end = self.begin+self.len;
        while line_end < source.len() && &source[line_end..line_end+1] != "\n" {
            line_end += 1;
        }
        let mut string = String::from(&source[line_start..line_end]);
        string.push('\n');
        for _ in line_start..self.begin {
            string.push(' ');
        }
        for _ in 0..self.len {
            string.push('^');
        }
        string
    }
}

pub fn tokenize(source: &str) -> Vec<Token> {
    // TODO the program "(" doesn't generate any tokens

    let mut tokens = vec![];

    if source.len() == 0 {
        return tokens;
    }

    let mut token_begin = 0;
    let mut token_len = 0;
    let mut i = 0;
    let mut not_first = false;
    while i < source.len() - 1 {
        if not_first {
            i += 1;
        } else {
            not_first = true;
        }

        let ch = source[i..i+1].chars().nth(0).unwrap();
        if token_len == 0 {
            token_begin = i;
        }

        // whitespace means end the current token
        if ch.is_whitespace() {
            if token_len != 0 {
                tokens.push(Token::new(token_begin, token_len));
                token_begin = 0;
                token_len = 0;
            }
            continue;
        }

        // enclose strings
        if token_len == 0 && ch == '"' {
            let quote_begin = i;
            i += 1;
            while i < source.len() && &source[i..i+1] != "\"" {
                // TI-84 basic does not have escape characters in strings
                /*if &source[i..i+1] == "\\" {
                    i += 1;
                }*/
                i += 1;
            }
            let quote_len = i + 1 - quote_begin;
            tokens.push(Token::new(quote_begin, quote_len));
            continue;
        }

        // singular symbols that are tokens in and of themselves
        if ch == '{' || ch == '}' || ch == '[' || ch == ']' || ch == '(' || ch == ')' || ch == ':' {
            if token_len != 0 {
                tokens.push(Token::new(token_begin, token_len));
                token_begin = 0;
                token_len = 0;
            }
            tokens.push(Token::new(i, 1));
            continue;
        }
        token_len += 1;
        if token_len == 1 {
            token_begin = i;
        }
    }
    tokens
}