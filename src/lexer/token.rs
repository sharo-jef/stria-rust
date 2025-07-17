#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    IntegerLiteral(IntegerType),
    FloatLiteral(FloatType),
    StringLiteral,
    BooleanLiteral,
    NullLiteral,
    
    // Identifiers and Keywords
    Identifier,
    Keyword(Keyword),
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Power,      // **
    
    // Comparison
    Equal,      // ==
    NotEqual,   // !=
    Less,       // <
    Greater,    // >
    LessEqual,  // <=
    GreaterEqual, // >=
    
    // Logical
    And,        // &&
    Or,         // ||
    Not,        // !
    
    // Bitwise
    BitAnd,     // &
    BitOr,      // |
    BitXor,     // ^
    BitNot,     // ~
    LeftShift,  // <<
    RightShift, // >>
    
    // Assignment
    Assign,     // =
    PlusAssign, // +=
    MinusAssign, // -=
    StarAssign, // *=
    SlashAssign, // /=
    PercentAssign, // %=
    
    // Punctuation
    LeftParen,  // (
    RightParen, // )
    LeftBrace,  // {
    RightBrace, // }
    LeftBracket, // [
    RightBracket, // ]
    Comma,      // ,
    Dot,        // .
    Colon,      // :
    Semicolon,  // ;
    Question,   // ?
    Arrow,      // ->
    Range,      // ..
    RangeInclusive, // ..=
    
    // Special
    Comment,
    Whitespace,
    Newline,
    Eof,
    
    // Schema directive
    SchemaDirective, // #schema
    
    // Spread operator
    Spread,     // ...
    
    // Null assertion
    NullAssert, // !
    
    // Type cast
    As,         // as
    
    // Type check
    Is,         // is
    
    // Step operator (for ranges)
    Step,       // step
    
    // Until operator
    Until,      // until
    
    // Down to operator
    DownTo,     // downTo
    
    // In operator
    In,         // in
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntegerType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FloatType {
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    // Control flow
    If,
    Else,
    Match,
    When,
    
    // Declarations
    Struct,
    Fun,
    Val,
    Var,
    Init,
    
    // Types
    String,
    Bool,
    
    // Literals
    True,
    False,
    Null,
    
    // Modifiers
    Private,
    Get,
    Repeated,
    Mixin,
    
    // Imports
    Use,
    
    // Schema
    Schema,
    
    // Error handling
    Error,
    
    // Special
    This,
    Return,
    
    // Infix
    Infix,
    
    // Annotation keywords
    Description,
    Name,
    Deprecated,
    Serialize,
    Flatten,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, value: String) -> Self {
        Self { kind, span, value }
    }
    
    pub fn is_keyword(&self) -> bool {
        matches!(self.kind, TokenKind::Keyword(_))
    }
    
    pub fn is_operator(&self) -> bool {
        matches!(self.kind, 
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash |
            TokenKind::Percent | TokenKind::Power | TokenKind::Equal | TokenKind::NotEqual |
            TokenKind::Less | TokenKind::Greater | TokenKind::LessEqual | TokenKind::GreaterEqual |
            TokenKind::And | TokenKind::Or | TokenKind::Not | TokenKind::BitAnd | TokenKind::BitOr |
            TokenKind::BitXor | TokenKind::BitNot | TokenKind::LeftShift | TokenKind::RightShift
        )
    }
    
    pub fn is_punctuation(&self) -> bool {
        matches!(self.kind,
            TokenKind::LeftParen | TokenKind::RightParen | TokenKind::LeftBrace | TokenKind::RightBrace |
            TokenKind::LeftBracket | TokenKind::RightBracket | TokenKind::Comma | TokenKind::Dot |
            TokenKind::Colon | TokenKind::Semicolon | TokenKind::Question | TokenKind::Arrow |
            TokenKind::Range | TokenKind::RangeInclusive
        )
    }
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }
    
    pub fn dummy() -> Self {
        Self::new(0, 0, 1, 1)
    }
}

impl IntegerType {
    pub fn from_suffix(suffix: &str) -> Option<Self> {
        match suffix {
            "i8" => Some(IntegerType::I8),
            "i16" => Some(IntegerType::I16),
            "i32" => Some(IntegerType::I32),
            "i64" => Some(IntegerType::I64),
            "u8" => Some(IntegerType::U8),
            "u16" => Some(IntegerType::U16),
            "u32" => Some(IntegerType::U32),
            "u64" => Some(IntegerType::U64),
            _ => None,
        }
    }
}

impl FloatType {
    pub fn from_suffix(suffix: &str) -> Option<Self> {
        match suffix {
            "f32" => Some(FloatType::F32),
            "f64" => Some(FloatType::F64),
            _ => None,
        }
    }
}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "if" => Some(Keyword::If),
            "else" => Some(Keyword::Else),
            "match" => Some(Keyword::Match),
            "when" => Some(Keyword::When),
            "struct" => Some(Keyword::Struct),
            "fun" => Some(Keyword::Fun),
            "val" => Some(Keyword::Val),
            "var" => Some(Keyword::Var),
            "init" => Some(Keyword::Init),
            "string" => Some(Keyword::String),
            "bool" => Some(Keyword::Bool),
            "true" => Some(Keyword::True),
            "false" => Some(Keyword::False),
            "null" => Some(Keyword::Null),
            "private" => Some(Keyword::Private),
            "get" => Some(Keyword::Get),
            "repeated" => Some(Keyword::Repeated),
            "mixin" => Some(Keyword::Mixin),
            "use" => Some(Keyword::Use),
            "schema" => Some(Keyword::Schema),
            "error" => Some(Keyword::Error),
            "this" => Some(Keyword::This),
            "return" => Some(Keyword::Return),
            "infix" => Some(Keyword::Infix),
            "as" => Some(Keyword::Name), // 'as' is handled separately
            "is" => Some(Keyword::Name), // 'is' is handled separately
            "step" => Some(Keyword::Name), // 'step' is handled separately
            "until" => Some(Keyword::Name), // 'until' is handled separately
            "downTo" => Some(Keyword::Name), // 'downTo' is handled separately
            "in" => Some(Keyword::Name), // 'in' is handled separately
            _ => None,
        }
    }
}
