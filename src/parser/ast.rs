#![allow(dead_code)]

use crate::lexer::Span;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    SchemaDirective(SchemaDirective),
    SchemaDeclaration(SchemaDeclaration),
    StructDeclaration(StructDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    VariableDeclaration(VariableDeclaration),
    UseStatement(UseStatement),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct SchemaDirective {
    pub path: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SchemaDeclaration {
    pub items: Vec<SchemaItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SchemaItem {
    pub property_name: String,
    pub type_name: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructDeclaration {
    pub name: String,
    pub init_methods: Vec<InitMethod>,
    pub properties: Vec<Property>,
    pub methods: Vec<Method>,
    pub mixins: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct InitMethod {
    pub parameters: Vec<Parameter>,
    pub body: Option<Block>,
    pub is_primary: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub default_value: Option<Expression>,
    pub is_optional: bool,
    pub is_repeated: bool,
    pub is_private: bool,
    pub repeated_constructors: Option<HashMap<String, String>>,
    pub annotations: Vec<Annotation>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Block,
    pub is_private: bool,
    pub is_getter: bool,
    pub is_infix: bool,
    pub annotations: Vec<Annotation>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeAnnotation>,
    pub body: Block,
    pub is_infix: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub default_value: Option<Expression>,
    pub is_this: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub initializer: Option<Expression>,
    pub is_mutable: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UseStatement {
    pub functions: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    pub type_expr: TypeExpression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeExpression {
    Primitive(PrimitiveType),
    List(Box<TypeExpression>),
    Optional(Box<TypeExpression>),
    Union(Vec<TypeExpression>),
    Identifier(String),
    Generic(String, Vec<TypeExpression>),
}

#[derive(Debug, Clone)]
pub enum PrimitiveType {
    String,
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    VariableDeclaration(VariableDeclaration),
    Assignment(Assignment),
    Return(Option<Expression>),
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub target: Expression,
    pub value: Expression,
    pub operator: AssignmentOperator,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AssignmentOperator {
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String, Span),
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),
    Call(Call),
    MemberAccess(MemberAccess),
    IndexAccess(IndexAccess),
    If(IfExpression),
    Match(MatchExpression),
    Lambda(Lambda),
    StructInstantiation(StructInstantiation),
    List(Vec<Expression>, Span),
    Range(RangeExpression),
    TypeCast(TypeCast),
    Block(Block),
    Error(String, Span),
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: LiteralValue,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    String(String),
    Integer(i64, IntegerType),
    Float(f64, FloatType),
    Boolean(bool),
    Null,
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

#[derive(Debug, Clone)]
pub struct BinaryOp {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,
    Range,
    RangeInclusive,
    In,
    Step,
    Until,
    DownTo,
    Is,
    As,
}

#[derive(Debug, Clone)]
pub struct UnaryOp {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not,
    Minus,
    Plus,
    BitNot,
    NullAssert,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub callee: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MemberAccess {
    pub object: Box<Expression>,
    pub member: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IndexAccess {
    pub object: Box<Expression>,
    pub index: Box<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfExpression {
    pub condition: Box<Expression>,
    pub then_branch: Box<Expression>,
    pub else_branch: Option<Box<Expression>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchExpression {
    pub value: Box<Expression>,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expression,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Identifier(String),
    TypeCheck(String, TypeExpression),
    Range(RangeExpression),
    Wildcard,
}

#[derive(Debug, Clone)]
pub struct Lambda {
    pub parameters: Vec<Parameter>,
    pub body: Box<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructInstantiation {
    pub name: String,
    pub arguments: Vec<Expression>,
    pub initializer: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RangeExpression {
    pub start: Box<Expression>,
    pub end: Box<Expression>,
    pub inclusive: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypeCast {
    pub expression: Box<Expression>,
    pub target_type: TypeAnnotation,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: String,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::Literal(lit) => lit.span.clone(),
            Expression::Identifier(_, span) => span.clone(),
            Expression::BinaryOp(op) => op.span.clone(),
            Expression::UnaryOp(op) => op.span.clone(),
            Expression::Call(call) => call.span.clone(),
            Expression::MemberAccess(acc) => acc.span.clone(),
            Expression::IndexAccess(acc) => acc.span.clone(),
            Expression::If(if_expr) => if_expr.span.clone(),
            Expression::Match(match_expr) => match_expr.span.clone(),
            Expression::Lambda(lambda) => lambda.span.clone(),
            Expression::StructInstantiation(inst) => inst.span.clone(),
            Expression::List(_, span) => span.clone(),
            Expression::Range(range) => range.span.clone(),
            Expression::TypeCast(cast) => cast.span.clone(),
            Expression::Block(block) => block.span.clone(),
            Expression::Error(_, span) => span.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer(IntegerType),
    Float(FloatType),
    String,
    Boolean,
    Null,
    Array(Box<Type>),
    List(Box<Type>),
    I32Range,
    F64Range,
    Struct(String),
    Function,
    Void,
    Optional(Box<Type>),
    Union(Vec<Type>),
}
