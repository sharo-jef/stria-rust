use crate::lexer::token::*;
use crate::error::{StriaResult, StriaError};

pub struct Tokenizer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self, input: &str) -> StriaResult<Vec<Token>> {
        self.input = input.to_string();
        self.position = 0;
        self.line = 1;
        self.column = 1;
        
        let mut tokens = Vec::new();
        
        while self.position < self.input.len() {
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                break;
            }
            
            let start_pos = self.position;
            let start_line = self.line;
            let start_col = self.column;
            
            let token = self.next_token()?;
            
            if let Some(token_kind) = token {
                let end_pos = self.position;
                let value = self.input[start_pos..end_pos].to_string();
                
                tokens.push(Token::new(
                    token_kind,
                    Span::new(start_pos, end_pos, start_line, start_col),
                    value,
                ));
            }
        }
        
        Ok(tokens)
    }
    
    fn next_token(&mut self) -> StriaResult<Option<TokenKind>> {
        let ch = self.current_char()?;
        
        match ch {
            // Comments
            '/' => {
                if self.peek_char() == Some('/') {
                    self.skip_line_comment();
                    return Ok(None);
                } else if self.peek_char() == Some('*') {
                    self.skip_block_comment()?;
                    return Ok(None);
                } else {
                    self.advance();
                    return Ok(Some(TokenKind::Slash));
                }
            }
            
            // String literals
            '"' => {
                self.advance(); // Skip opening quote
                while self.position < self.input.len() && self.current_char()? != '"' {
                    if self.current_char()? == '\\' {
                        self.advance(); // Skip escape character
                        if self.position < self.input.len() {
                            self.advance(); // Skip escaped character
                        }
                    } else {
                        self.advance();
                    }
                }
                if self.position < self.input.len() {
                    self.advance(); // Skip closing quote
                }
                return Ok(Some(TokenKind::StringLiteral));
            }
            
            '\'' => {
                self.advance(); // Skip opening quote
                while self.position < self.input.len() && self.current_char()? != '\'' {
                    if self.current_char()? == '\\' {
                        self.advance(); // Skip escape character
                        if self.position < self.input.len() {
                            self.advance(); // Skip escaped character
                        }
                    } else {
                        self.advance();
                    }
                }
                if self.position < self.input.len() {
                    self.advance(); // Skip closing quote
                }
                return Ok(Some(TokenKind::StringLiteral));
            }
            
            // Numbers
            '0'..='9' => {
                return Ok(Some(self.read_number()));
            }
            
            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                return Ok(Some(self.read_identifier()));
            }
            
            // Two-character operators
            '=' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    return Ok(Some(TokenKind::Equal));
                } else if self.peek_char() == Some('>') {
                    self.advance();
                    return Ok(Some(TokenKind::Arrow));
                } else {
                    return Ok(Some(TokenKind::Assign));
                }
            }
            
            '!' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    return Ok(Some(TokenKind::NotEqual));
                } else {
                    return Ok(Some(TokenKind::Not));
                }
            }
            
            '<' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    return Ok(Some(TokenKind::LessEqual));
                } else if self.peek_char() == Some('<') {
                    self.advance();
                    return Ok(Some(TokenKind::LeftShift));
                } else {
                    return Ok(Some(TokenKind::Less));
                }
            }
            
            '>' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    return Ok(Some(TokenKind::GreaterEqual));
                } else if self.peek_char() == Some('>') {
                    self.advance();
                    return Ok(Some(TokenKind::RightShift));
                } else {
                    return Ok(Some(TokenKind::Greater));
                }
            }
            
            '&' => {
                self.advance();
                if self.peek_char() == Some('&') {
                    self.advance();
                    return Ok(Some(TokenKind::And));
                } else {
                    return Ok(Some(TokenKind::BitAnd));
                }
            }
            
            '|' => {
                self.advance();
                if self.peek_char() == Some('|') {
                    self.advance();
                    return Ok(Some(TokenKind::Or));
                } else {
                    return Ok(Some(TokenKind::BitOr));
                }
            }
            
            '+' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    return Ok(Some(TokenKind::PlusAssign));
                } else {
                    return Ok(Some(TokenKind::Plus));
                }
            }
            
            '-' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    return Ok(Some(TokenKind::MinusAssign));
                } else {
                    return Ok(Some(TokenKind::Minus));
                }
            }
            
            '*' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    return Ok(Some(TokenKind::StarAssign));
                } else if self.peek_char() == Some('*') {
                    self.advance();
                    return Ok(Some(TokenKind::Power));
                } else {
                    return Ok(Some(TokenKind::Star));
                }
            }
            
            '%' => {
                self.advance();
                if self.peek_char() == Some('=') {
                    self.advance();
                    return Ok(Some(TokenKind::PercentAssign));
                } else {
                    return Ok(Some(TokenKind::Percent));
                }
            }
            
            '.' => {
                self.advance();
                if self.peek_char() == Some('.') {
                    self.advance();
                    if self.peek_char() == Some('.') {
                        self.advance();
                        return Ok(Some(TokenKind::Spread));
                    } else if self.peek_char() == Some('=') {
                        self.advance();
                        return Ok(Some(TokenKind::RangeInclusive));
                    } else {
                        return Ok(Some(TokenKind::Range));
                    }
                } else {
                    return Ok(Some(TokenKind::Dot));
                }
            }
            
            ':' => {
                self.advance();
                return Ok(Some(TokenKind::Colon));
            }
            
            // Single character tokens
            '(' => { self.advance(); Ok(Some(TokenKind::LeftParen)) }
            ')' => { self.advance(); Ok(Some(TokenKind::RightParen)) }
            '[' => { self.advance(); Ok(Some(TokenKind::LeftBracket)) }
            ']' => { self.advance(); Ok(Some(TokenKind::RightBracket)) }
            '{' => { self.advance(); Ok(Some(TokenKind::LeftBrace)) }
            '}' => { self.advance(); Ok(Some(TokenKind::RightBrace)) }
            ',' => { self.advance(); Ok(Some(TokenKind::Comma)) }
            ';' => { self.advance(); Ok(Some(TokenKind::Semicolon)) }
            '?' => { self.advance(); Ok(Some(TokenKind::Question)) }
            '^' => { self.advance(); Ok(Some(TokenKind::BitXor)) }
            '~' => { self.advance(); Ok(Some(TokenKind::BitNot)) }
            
            _ => {
                self.advance();
                Ok(None) // Skip unknown characters
            }
        }
    }
    
    fn read_number(&mut self) -> TokenKind {
        let mut is_float = false;
        
        // Read integer part
        while self.position < self.input.len() && self.current_char().unwrap_or('\0').is_ascii_digit() {
            self.advance();
        }
        
        // Check for decimal point
        if self.position < self.input.len() && self.current_char().unwrap_or('\0') == '.' {
            let next_char = self.peek_char().unwrap_or('\0');
            if next_char.is_ascii_digit() {
                is_float = true;
                self.advance(); // Skip '.'
                while self.position < self.input.len() && self.current_char().unwrap_or('\0').is_ascii_digit() {
                    self.advance();
                }
            }
        }
        
        // Check for exponent
        if self.position < self.input.len() {
            let ch = self.current_char().unwrap_or('\0');
            if ch == 'e' || ch == 'E' {
                is_float = true;
                self.advance();
                
                // Optional sign
                if self.position < self.input.len() {
                    let sign = self.current_char().unwrap_or('\0');
                    if sign == '+' || sign == '-' {
                        self.advance();
                    }
                }
                
                // Exponent digits
                while self.position < self.input.len() && self.current_char().unwrap_or('\0').is_ascii_digit() {
                    self.advance();
                }
            }
        }
        
        // Check for type suffix
        if self.position < self.input.len() {
            let remaining = &self.input[self.position..];
            
            if remaining.starts_with("f32") {
                self.position += 3;
                return TokenKind::FloatLiteral(FloatType::F32);
            } else if remaining.starts_with("f64") {
                self.position += 3;
                return TokenKind::FloatLiteral(FloatType::F64);
            } else if remaining.starts_with("i8") {
                self.position += 2;
                return TokenKind::IntegerLiteral(IntegerType::I8);
            } else if remaining.starts_with("i16") {
                self.position += 3;
                return TokenKind::IntegerLiteral(IntegerType::I16);
            } else if remaining.starts_with("i32") {
                self.position += 3;
                return TokenKind::IntegerLiteral(IntegerType::I32);
            } else if remaining.starts_with("i64") {
                self.position += 3;
                return TokenKind::IntegerLiteral(IntegerType::I64);
            } else if remaining.starts_with("u8") {
                self.position += 2;
                return TokenKind::IntegerLiteral(IntegerType::U8);
            } else if remaining.starts_with("u16") {
                self.position += 3;
                return TokenKind::IntegerLiteral(IntegerType::U16);
            } else if remaining.starts_with("u32") {
                self.position += 3;
                return TokenKind::IntegerLiteral(IntegerType::U32);
            } else if remaining.starts_with("u64") {
                self.position += 3;
                return TokenKind::IntegerLiteral(IntegerType::U64);
            }
        }
        
        if is_float {
            TokenKind::FloatLiteral(FloatType::F64)
        } else {
            TokenKind::IntegerLiteral(IntegerType::I32)
        }
    }
    
    fn read_identifier(&mut self) -> TokenKind {
        let start = self.position;
        
        while self.position < self.input.len() {
            let ch = self.current_char().unwrap_or('\0');
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        let identifier = &self.input[start..self.position];
        
        // Check for keywords
        match identifier {
            "true" => TokenKind::BooleanLiteral,
            "false" => TokenKind::BooleanLiteral,
            "null" => TokenKind::NullLiteral,
            "if" => TokenKind::Keyword(Keyword::If),
            "else" => TokenKind::Keyword(Keyword::Else),
            "match" => TokenKind::Keyword(Keyword::Match),
            "when" => TokenKind::Keyword(Keyword::When),
            "struct" => TokenKind::Keyword(Keyword::Struct),
            "function" | "fun" => TokenKind::Keyword(Keyword::Fun),
            "val" => TokenKind::Keyword(Keyword::Val),
            "var" => TokenKind::Keyword(Keyword::Var),
            "init" => TokenKind::Keyword(Keyword::Init),
            "string" => TokenKind::Keyword(Keyword::String),
            "bool" => TokenKind::Keyword(Keyword::Bool),
            "private" => TokenKind::Keyword(Keyword::Private),
            "get" => TokenKind::Keyword(Keyword::Get),
            "repeated" => TokenKind::Keyword(Keyword::Repeated),
            "mixin" => TokenKind::Keyword(Keyword::Mixin),
            "use" => TokenKind::Keyword(Keyword::Use),
            "schema" => TokenKind::Keyword(Keyword::Schema),
            "error" => TokenKind::Keyword(Keyword::Error),
            "this" => TokenKind::Keyword(Keyword::This),
            "return" => TokenKind::Keyword(Keyword::Return),
            "infix" => TokenKind::Keyword(Keyword::Infix),
            "as" => TokenKind::As,
            "is" => TokenKind::Is,
            "step" => TokenKind::Step,
            "until" => TokenKind::Until,
            "downTo" => TokenKind::DownTo,
            "in" => TokenKind::In,
            _ => TokenKind::Identifier,
        }
    }
    
    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() {
            let ch = self.current_char().unwrap_or('\0');
            if ch.is_whitespace() {
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
                self.position += 1;
            } else {
                break;
            }
        }
    }
    
    fn skip_line_comment(&mut self) {
        while self.position < self.input.len() {
            if self.current_char().unwrap_or('\0') == '\n' {
                break;
            }
            self.advance();
        }
    }
    
    fn skip_block_comment(&mut self) -> StriaResult<()> {
        self.advance(); // Skip '/'
        self.advance(); // Skip '*'
        
        while self.position < self.input.len() {
            if self.current_char()? == '*' && self.peek_char() == Some('/') {
                self.advance(); // Skip '*'
                self.advance(); // Skip '/'
                return Ok(());
            }
            self.advance();
        }
        
        Err(StriaError::lexer("Unterminated block comment".to_string()))
    }
    
    fn current_char(&self) -> StriaResult<char> {
        if self.position >= self.input.len() {
            return Err(StriaError::lexer("Unexpected end of input".to_string()));
        }
        Ok(self.input.chars().nth(self.position).unwrap())
    }
    
    fn peek_char(&self) -> Option<char> {
        if self.position + 1 >= self.input.len() {
            None
        } else {
            self.input.chars().nth(self.position + 1)
        }
    }
    
    fn advance(&mut self) {
        if self.position < self.input.len() {
            let ch = self.current_char().unwrap_or('\0');
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }
}
