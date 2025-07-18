use crate::error::{StriaError, StriaResult};
use crate::lexer::{Keyword, Token, TokenKind};
use crate::parser::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    eof_token: Token,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let eof_token = Token {
            kind: TokenKind::Eof,
            span: crate::lexer::Span::new(0, 0, 1, 1),
            value: String::new(),
        };

        Self {
            tokens,
            current: 0,
            eof_token,
        }
    }

    pub fn parse(&mut self) -> StriaResult<Program> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            if let Some(item) = self.parse_item()? {
                items.push(item);
            }
        }

        Ok(Program { items })
    }

    fn parse_item(&mut self) -> StriaResult<Option<Item>> {
        if self.is_at_end() {
            return Ok(None);
        }

        match &self.peek().kind {
            TokenKind::SchemaDirective => {
                let schema_directive = self.parse_schema_directive()?;
                Ok(Some(Item::SchemaDirective(schema_directive)))
            }
            TokenKind::Keyword(Keyword::Schema) => {
                let schema = self.parse_schema_declaration()?;
                Ok(Some(Item::SchemaDeclaration(schema)))
            }
            TokenKind::Keyword(Keyword::Struct) => {
                let struct_decl = self.parse_struct_declaration()?;
                Ok(Some(Item::StructDeclaration(struct_decl)))
            }
            TokenKind::Keyword(Keyword::Fun) => {
                let func_decl = self.parse_function_declaration()?;
                Ok(Some(Item::FunctionDeclaration(func_decl)))
            }
            TokenKind::Keyword(Keyword::Val) | TokenKind::Keyword(Keyword::Var) => {
                let var_decl = self.parse_variable_declaration()?;
                Ok(Some(Item::VariableDeclaration(var_decl)))
            }
            TokenKind::Keyword(Keyword::Use) => {
                let use_stmt = self.parse_use_statement()?;
                Ok(Some(Item::UseStatement(use_stmt)))
            }
            _ => {
                // Try to parse as expression
                let expr = self.parse_expression()?;
                Ok(Some(Item::Expression(expr)))
            }
        }
    }

    fn parse_schema_declaration(&mut self) -> StriaResult<SchemaDeclaration> {
        let start = self.advance().span.start; // consume 'schema'

        let mut items = Vec::new();

        // Expect opening brace
        if !self.match_token(&TokenKind::LeftBrace) {
            return Err(StriaError::parser(
                "Expected '{' after 'schema'".to_string(),
            ));
        }

        // Parse schema properties
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            // Parse property name or struct name
            if let TokenKind::Identifier = self.peek().kind {
                let first_name = self.advance().value.clone();
                let first_token_span = self.previous().span.clone();

                // Check if this is the old format (just struct names) or new format (property: Type)
                if self.check(&TokenKind::Colon) {
                    // New format: property: Type
                    self.advance(); // consume ':'

                    // Parse type name
                    if let TokenKind::Identifier = self.peek().kind {
                        let type_token = self.advance();
                        items.push(crate::parser::ast::SchemaItem {
                            property_name: first_name,
                            type_name: type_token.value.clone(),
                            span: type_token.span.clone(),
                        });
                    } else {
                        return Err(StriaError::parser(
                            "Expected type name after ':' in schema".to_string(),
                        ));
                    }
                } else {
                    // Old format: just struct names
                    items.push(crate::parser::ast::SchemaItem {
                        property_name: first_name.clone(), // Use struct name as property name
                        type_name: first_name,
                        span: first_token_span,
                    });
                }
            } else {
                return Err(StriaError::parser(
                    "Expected property name or struct name in schema".to_string(),
                ));
            }

            // Optional comma
            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }

        // Expect closing brace
        if !self.match_token(&TokenKind::RightBrace) {
            return Err(StriaError::parser(
                "Expected '}' after schema body".to_string(),
            ));
        }

        let end = self.previous().span.end;

        Ok(SchemaDeclaration {
            items,
            span: crate::lexer::Span::new(start, end, 1, 1),
        })
    }

    fn parse_struct_declaration(&mut self) -> StriaResult<StructDeclaration> {
        let start = self.advance().span.start; // consume 'struct'

        let name = if let TokenKind::Identifier = self.peek().kind {
            self.advance().value.clone()
        } else {
            return Err(StriaError::parser("Expected struct name".to_string()));
        };

        if !self.match_token(&TokenKind::LeftBrace) {
            return Err(StriaError::parser(
                "Expected '{' after struct name".to_string(),
            ));
        }

        let mut init_methods = Vec::new();
        let mut properties = Vec::new();
        let mut methods = Vec::new();
        let mut mixins = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.match_token(&TokenKind::Keyword(Keyword::Init)) {
                init_methods.push(self.parse_init_method()?);
            } else if self.match_token(&TokenKind::Keyword(Keyword::Fun)) {
                methods.push(self.parse_method()?);
            } else if self.match_token(&TokenKind::Keyword(Keyword::Mixin)) {
                if let TokenKind::Identifier = self.peek().kind {
                    mixins.push(self.advance().value.clone());
                }
            } else {
                properties.push(self.parse_property()?);
            }
        }

        if !self.match_token(&TokenKind::RightBrace) {
            return Err(StriaError::parser(
                "Expected '}' after struct body".to_string(),
            ));
        }

        let end = self.previous().span.end;

        Ok(StructDeclaration {
            name,
            init_methods,
            properties,
            methods,
            mixins,
            span: crate::lexer::Span::new(start, end, 1, 1),
        })
    }

    fn parse_init_method(&mut self) -> StriaResult<InitMethod> {
        let start = self.previous().span.start;

        let mut parameters = Vec::new();

        if self.match_token(&TokenKind::LeftParen) {
            while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                parameters.push(self.parse_parameter()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }

            if !self.match_token(&TokenKind::RightParen) {
                return Err(StriaError::parser(
                    "Expected ')' after parameters".to_string(),
                ));
            }
        }

        let body = if self.check(&TokenKind::LeftBrace) {
            Some(self.parse_block()?)
        } else {
            None
        };

        let end = self.previous().span.end;

        Ok(InitMethod {
            parameters,
            body,
            is_primary: true,
            span: crate::lexer::Span::new(start, end, 1, 1),
        })
    }

    fn parse_method(&mut self) -> StriaResult<Method> {
        let start = self.previous().span.start;

        let is_getter = self.match_token(&TokenKind::Keyword(Keyword::Get));
        let is_infix = self.match_token(&TokenKind::Keyword(Keyword::Infix));
        let is_private = self.match_token(&TokenKind::Keyword(Keyword::Private));

        let name = if let TokenKind::Identifier = self.peek().kind {
            self.advance().value.clone()
        } else {
            return Err(StriaError::parser("Expected method name".to_string()));
        };

        let mut parameters = Vec::new();

        if self.match_token(&TokenKind::LeftParen) {
            while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                parameters.push(self.parse_parameter()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }

            if !self.match_token(&TokenKind::RightParen) {
                return Err(StriaError::parser(
                    "Expected ')' after parameters".to_string(),
                ));
            }
        }

        let return_type = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        let end = self.previous().span.end;

        Ok(Method {
            name,
            parameters,
            return_type,
            body,
            is_private,
            is_getter,
            is_infix,
            annotations: Vec::new(),
            span: crate::lexer::Span::new(start, end, 1, 1),
        })
    }

    fn parse_property(&mut self) -> StriaResult<Property> {
        let start = self.peek().span.start;

        let is_repeated = self.match_token(&TokenKind::Keyword(Keyword::Repeated));
        let is_private = self.match_token(&TokenKind::Keyword(Keyword::Private));

        let name = if let TokenKind::Identifier = self.peek().kind {
            self.advance().value.clone()
        } else {
            return Err(StriaError::parser("Expected property name".to_string()));
        };

        let type_annotation = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let default_value = if self.match_token(&TokenKind::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end = self.previous().span.end;

        Ok(Property {
            name,
            type_annotation,
            default_value,
            is_optional: false,
            is_repeated,
            is_private,
            repeated_constructors: None,
            annotations: Vec::new(),
            span: crate::lexer::Span::new(start, end, 1, 1),
        })
    }

    fn parse_function_declaration(&mut self) -> StriaResult<FunctionDeclaration> {
        let start = self.advance().span.start; // consume 'fun'

        let is_infix = self.match_token(&TokenKind::Keyword(Keyword::Infix));

        let name = if let TokenKind::Identifier = self.peek().kind {
            self.advance().value.clone()
        } else {
            return Err(StriaError::parser("Expected function name".to_string()));
        };

        let mut parameters = Vec::new();

        if self.match_token(&TokenKind::LeftParen) {
            while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                parameters.push(self.parse_parameter()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }

            if !self.match_token(&TokenKind::RightParen) {
                return Err(StriaError::parser(
                    "Expected ')' after parameters".to_string(),
                ));
            }
        }

        let return_type = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        let end = self.previous().span.end;

        Ok(FunctionDeclaration {
            name,
            parameters,
            return_type,
            body,
            is_infix,
            span: crate::lexer::Span::new(start, end, 1, 1),
        })
    }

    fn parse_variable_declaration(&mut self) -> StriaResult<VariableDeclaration> {
        let start = self.peek().span.start;
        let is_mutable = self.match_token(&TokenKind::Keyword(Keyword::Var));

        if !is_mutable && !self.match_token(&TokenKind::Keyword(Keyword::Val)) {
            return Err(StriaError::parser("Expected 'val' or 'var'".to_string()));
        }

        let name = if let TokenKind::Identifier = self.peek().kind {
            self.advance().value.clone()
        } else {
            return Err(StriaError::parser("Expected variable name".to_string()));
        };

        let type_annotation = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let initializer = if self.match_token(&TokenKind::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end = self.previous().span.end;

        Ok(VariableDeclaration {
            name,
            type_annotation,
            initializer,
            is_mutable,
            span: crate::lexer::Span::new(start, end, 1, 1),
        })
    }

    fn parse_use_statement(&mut self) -> StriaResult<UseStatement> {
        let start = self.advance().span.start; // consume 'use'

        let mut functions = Vec::new();

        if let TokenKind::Identifier = self.peek().kind {
            functions.push(self.advance().value.clone());
        }

        let end = self.previous().span.end;

        Ok(UseStatement {
            functions,
            span: crate::lexer::Span::new(start, end, 1, 1),
        })
    }

    fn parse_parameter(&mut self) -> StriaResult<Parameter> {
        let start = self.peek().span.start;

        let is_this = self.match_token(&TokenKind::Keyword(Keyword::This));

        let name = if is_this {
            // If we have 'this', expect '.' followed by identifier
            if !self.match_token(&TokenKind::Dot) {
                return Err(StriaError::parser("Expected '.' after 'this'".to_string()));
            }

            if let TokenKind::Identifier = self.peek().kind {
                self.advance().value.clone()
            } else {
                return Err(StriaError::parser(
                    "Expected property name after 'this.'".to_string(),
                ));
            }
        } else {
            // Regular parameter name
            if let TokenKind::Identifier = self.peek().kind {
                self.advance().value.clone()
            } else {
                return Err(StriaError::parser("Expected parameter name".to_string()));
            }
        };

        let type_annotation = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let default_value = if self.match_token(&TokenKind::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end = self.previous().span.end;

        Ok(Parameter {
            name,
            type_annotation,
            default_value,
            is_this,
            span: crate::lexer::Span::new(start, end, 1, 1),
        })
    }

    fn parse_type_annotation(&mut self) -> StriaResult<TypeAnnotation> {
        let start = self.peek().span.start;
        let type_expr = self.parse_type_expression()?;
        let end = self.previous().span.end;

        Ok(TypeAnnotation {
            type_expr,
            span: crate::lexer::Span::new(start, end, 1, 1),
        })
    }

    fn parse_type_expression(&mut self) -> StriaResult<TypeExpression> {
        match &self.peek().kind {
            TokenKind::Keyword(Keyword::String) => {
                self.advance();
                Ok(TypeExpression::Primitive(PrimitiveType::String))
            }
            TokenKind::Keyword(Keyword::Bool) => {
                self.advance();
                Ok(TypeExpression::Primitive(PrimitiveType::Bool))
            }
            TokenKind::Identifier => {
                let name = self.advance().value.clone();
                Ok(TypeExpression::Identifier(name))
            }
            _ => Err(StriaError::parser("Expected type expression".to_string())),
        }
    }

    fn parse_block(&mut self) -> StriaResult<Block> {
        let start = self.peek().span.start;

        if !self.match_token(&TokenKind::LeftBrace) {
            return Err(StriaError::parser("Expected '{'".to_string()));
        }

        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        if !self.match_token(&TokenKind::RightBrace) {
            return Err(StriaError::parser("Expected '}'".to_string()));
        }

        let end = self.previous().span.end;

        Ok(Block {
            statements,
            span: crate::lexer::Span::new(start, end, 1, 1),
        })
    }

    fn parse_statement(&mut self) -> StriaResult<Statement> {
        if self.match_token(&TokenKind::Keyword(Keyword::Return)) {
            let expr = if self.check(&TokenKind::Semicolon) || self.check(&TokenKind::RightBrace) {
                None
            } else {
                Some(self.parse_expression()?)
            };
            Ok(Statement::Return(expr))
        } else if self.check(&TokenKind::Keyword(Keyword::Val))
            || self.check(&TokenKind::Keyword(Keyword::Var))
        {
            let var_decl = self.parse_variable_declaration()?;
            Ok(Statement::VariableDeclaration(var_decl))
        } else {
            let expr = self.parse_expression()?;
            Ok(Statement::Expression(expr))
        }
    }

    fn parse_expression(&mut self) -> StriaResult<Expression> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> StriaResult<Expression> {
        let expr = self.parse_logical_or()?;

        if self.match_token(&TokenKind::Assign) {
            let value = self.parse_assignment()?;
            let span = crate::lexer::Span::new(0, 0, 1, 1); // TODO: proper span

            return Ok(Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator: BinaryOperator::Equal, // Using Equal as assignment for now
                right: Box::new(value),
                span,
            }));
        }

        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> StriaResult<Expression> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(&TokenKind::Or) {
            let right = self.parse_logical_and()?;
            let span = crate::lexer::Span::new(0, 0, 1, 1); // TODO: proper span

            expr = Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator: BinaryOperator::Or,
                right: Box::new(right),
                span,
            });
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> StriaResult<Expression> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&TokenKind::And) {
            let right = self.parse_equality()?;
            let span = crate::lexer::Span::new(0, 0, 1, 1); // TODO: proper span

            expr = Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator: BinaryOperator::And,
                right: Box::new(right),
                span,
            });
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> StriaResult<Expression> {
        let mut expr = self.parse_comparison()?;

        while self.match_tokens(&[TokenKind::Equal, TokenKind::NotEqual]) {
            let operator = match self.previous().kind {
                TokenKind::Equal => BinaryOperator::Equal,
                TokenKind::NotEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };

            let right = self.parse_comparison()?;
            let span = crate::lexer::Span::new(0, 0, 1, 1); // TODO: proper span

            expr = Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            });
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> StriaResult<Expression> {
        let mut expr = self.parse_cast()?;

        while self.match_tokens(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = match self.previous().kind {
                TokenKind::Greater => BinaryOperator::Greater,
                TokenKind::GreaterEqual => BinaryOperator::GreaterEqual,
                TokenKind::Less => BinaryOperator::Less,
                TokenKind::LessEqual => BinaryOperator::LessEqual,
                _ => unreachable!(),
            };

            let right = self.parse_cast()?;
            let span = crate::lexer::Span::new(0, 0, 1, 1); // TODO: proper span

            expr = Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            });
        }

        Ok(expr)
    }

    fn parse_cast(&mut self) -> StriaResult<Expression> {
        let mut expr = self.parse_term()?;

        while self.match_token(&TokenKind::As) {
            let target_type = self.parse_type_annotation()?;
            let span = crate::lexer::Span::new(0, 0, 1, 1); // TODO: proper span

            expr = Expression::TypeCast(TypeCast {
                expression: Box::new(expr),
                target_type,
                span,
            });
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> StriaResult<Expression> {
        let mut expr = self.parse_factor()?;

        while self.match_tokens(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = match self.previous().kind {
                TokenKind::Minus => BinaryOperator::Subtract,
                TokenKind::Plus => BinaryOperator::Add,
                _ => unreachable!(),
            };

            let right = self.parse_factor()?;
            let span = crate::lexer::Span::new(0, 0, 1, 1); // TODO: proper span

            expr = Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            });
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> StriaResult<Expression> {
        let mut expr = self.parse_unary()?;

        while self.match_tokens(&[TokenKind::Slash, TokenKind::Star, TokenKind::Percent]) {
            let operator = match self.previous().kind {
                TokenKind::Slash => BinaryOperator::Divide,
                TokenKind::Star => BinaryOperator::Multiply,
                TokenKind::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };

            let right = self.parse_unary()?;
            let span = crate::lexer::Span::new(0, 0, 1, 1); // TODO: proper span

            expr = Expression::BinaryOp(BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                span,
            });
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> StriaResult<Expression> {
        if self.match_tokens(&[TokenKind::Not, TokenKind::Minus]) {
            let operator = match self.previous().kind {
                TokenKind::Not => UnaryOperator::Not,
                TokenKind::Minus => UnaryOperator::Minus,
                _ => unreachable!(),
            };

            let right = self.parse_unary()?;
            let span = crate::lexer::Span::new(0, 0, 1, 1); // TODO: proper span

            return Ok(Expression::UnaryOp(UnaryOp {
                operator,
                operand: Box::new(right),
                span,
            }));
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> StriaResult<Expression> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&TokenKind::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&TokenKind::Dot) {
                let name = if let TokenKind::Identifier = self.peek().kind {
                    self.advance().value.clone()
                } else {
                    return Err(StriaError::parser(
                        "Expected property name after '.'".to_string(),
                    ));
                };

                let span = crate::lexer::Span::new(0, 0, 1, 1); // TODO: proper span

                expr = Expression::MemberAccess(MemberAccess {
                    object: Box::new(expr),
                    member: name,
                    span,
                });
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expression) -> StriaResult<Expression> {
        let mut arguments = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                arguments.push(self.parse_expression()?);

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        if !self.match_token(&TokenKind::RightParen) {
            return Err(StriaError::parser(
                "Expected ')' after arguments".to_string(),
            ));
        }

        let span = crate::lexer::Span::new(0, 0, 1, 1); // TODO: proper span

        Ok(Expression::Call(Call {
            callee: Box::new(callee),
            arguments,
            span,
        }))
    }

    fn parse_primary(&mut self) -> StriaResult<Expression> {
        match &self.peek().kind {
            TokenKind::BooleanLiteral => {
                let token = self.advance();
                let value = token.value == "true";
                Ok(Expression::Literal(Literal {
                    value: LiteralValue::Boolean(value),
                    span: token.span.clone(),
                }))
            }

            TokenKind::NullLiteral => {
                let token = self.advance();
                Ok(Expression::Literal(Literal {
                    value: LiteralValue::Null,
                    span: token.span.clone(),
                }))
            }

            TokenKind::IntegerLiteral(int_type) => {
                let int_type = int_type.clone();
                let token = self.advance();
                let value = token.value.parse::<i64>().unwrap_or(0);
                let ast_type = match int_type {
                    crate::lexer::IntegerType::I8 => IntegerType::I8,
                    crate::lexer::IntegerType::I16 => IntegerType::I16,
                    crate::lexer::IntegerType::I32 => IntegerType::I32,
                    crate::lexer::IntegerType::I64 => IntegerType::I64,
                    crate::lexer::IntegerType::U8 => IntegerType::U8,
                    crate::lexer::IntegerType::U16 => IntegerType::U16,
                    crate::lexer::IntegerType::U32 => IntegerType::U32,
                    crate::lexer::IntegerType::U64 => IntegerType::U64,
                };
                Ok(Expression::Literal(Literal {
                    value: LiteralValue::Integer(value, ast_type),
                    span: token.span.clone(),
                }))
            }

            TokenKind::FloatLiteral(float_type) => {
                let float_type = float_type.clone();
                let token = self.advance();
                let value = token.value.parse::<f64>().unwrap_or(0.0);
                let ast_type = match float_type {
                    crate::lexer::FloatType::F32 => FloatType::F32,
                    crate::lexer::FloatType::F64 => FloatType::F64,
                };
                Ok(Expression::Literal(Literal {
                    value: LiteralValue::Float(value, ast_type),
                    span: token.span.clone(),
                }))
            }

            TokenKind::StringLiteral => {
                let token = self.advance();
                Ok(Expression::Literal(Literal {
                    value: LiteralValue::String(token.value.clone()),
                    span: token.span.clone(),
                }))
            }

            TokenKind::Identifier => {
                let token = self.advance();
                let name = token.value.clone();
                let span = token.span.clone();

                // Check if this is a struct instantiation with optional parentheses
                if self.check(&TokenKind::LeftParen) {
                    let left_paren_span = self.advance().span.clone(); // consume '(' and remember its position

                    let mut arguments = Vec::new();

                    // Parse arguments inside parentheses
                    while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                        arguments.push(self.parse_expression()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }

                    if !self.match_token(&TokenKind::RightParen) {
                        // Calculate the error position: after the last argument or after the opening '('
                        let error_span = if let Some(last_arg) = arguments.last() {
                            // Get the span of the last argument and point to the position after it
                            let last_span = match last_arg {
                                Expression::Literal(lit) => &lit.span,
                                Expression::Identifier(_, span) => span,
                                Expression::Call(call) => &call.span,
                                Expression::StructInstantiation(struct_inst) => &struct_inst.span,
                                Expression::BinaryOp(binary) => &binary.span,
                                Expression::UnaryOp(unary) => &unary.span,
                                Expression::MemberAccess(member) => &member.span,
                                Expression::IndexAccess(index) => &index.span,
                                Expression::If(if_expr) => &if_expr.span,
                                Expression::Match(match_expr) => &match_expr.span,
                                Expression::Lambda(lambda) => &lambda.span,
                                Expression::List(_, span) => span,
                                Expression::Range(range) => &range.span,
                                Expression::TypeCast(cast) => &cast.span,
                                Expression::Block(block) => &block.span,
                                Expression::Error(_, span) => span,
                            };
                            // Create a span that points to the end of the last argument
                            crate::lexer::Span::new(
                                last_span.end,
                                last_span.end,
                                last_span.line,
                                last_span.column + (last_span.end - last_span.start),
                            )
                        } else {
                            // If no arguments, point to the position after the opening '('
                            crate::lexer::Span::new(
                                left_paren_span.end,
                                left_paren_span.end,
                                left_paren_span.line,
                                left_paren_span.column + 1,
                            )
                        };

                        return Err(StriaError::parser_with_span(
                            "Expected ')' after expression".to_string(),
                            error_span,
                        ));
                    }

                    // After (), check for { (struct instantiation) or nothing (function call)
                    if self.check(&TokenKind::LeftBrace) {
                        self.advance(); // consume '{'
                        let mut field_assignments = Vec::new();

                        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                            // Parse field name
                            if let TokenKind::Identifier = self.peek().kind {
                                let field_name = self.advance().value.clone();

                                // Expect '='
                                if !self.match_token(&TokenKind::Assign) {
                                    let current_token = self.peek().clone();
                                    return Err(StriaError::parser_with_span_and_help(
                                        format!(
                                            "Expected '=' after field name '{}', found '{}'",
                                            field_name, current_token.value
                                        ),
                                        current_token.span,
                                        "struct fields must be assigned with '=' operator",
                                    ));
                                }

                                // Parse field value
                                let field_value = self.parse_expression()?;

                                // Create an assignment expression for this field
                                field_assignments
                                    .push(Expression::Identifier(field_name, span.clone()));
                                field_assignments.push(field_value);

                                // Handle comma, semicolon, or end - allow newlines as field separators
                                if self.check(&TokenKind::Comma)
                                    || self.check(&TokenKind::Semicolon)
                                {
                                    self.advance(); // consume ',' or ';'
                                } else if self.check(&TokenKind::RightBrace) {
                                    // End of struct, will be handled by outer loop
                                } else {
                                    // Allow implicit field separation (newlines act as separators)
                                }
                            } else {
                                let current_token = self.peek().clone();
                                return Err(StriaError::parser_with_span_and_help(
                                    format!("Expected field name, found '{}'", current_token.value),
                                    current_token.span,
                                    "field names must be valid identifiers",
                                ));
                            }
                        }

                        if !self.match_token(&TokenKind::RightBrace) {
                            let current_token = self.peek().clone();
                            return Err(StriaError::parser_with_span(
                                format!(
                                    "Expected '}}' after struct fields, found '{}'",
                                    current_token.value
                                ),
                                current_token.span,
                            ));
                        }

                        // Combine constructor arguments with field assignments
                        let mut all_arguments = arguments;
                        all_arguments.extend(field_assignments);

                        Ok(Expression::StructInstantiation(StructInstantiation {
                            name,
                            arguments: all_arguments,
                            initializer: None,
                            span,
                        }))
                    } else {
                        // This is either a struct instantiation without {} or a function call
                        if arguments.is_empty() {
                            // Empty parentheses - could be struct instantiation or function call
                            // For now, treat as function call since no {} follows
                            Ok(Expression::Call(Call {
                                callee: Box::new(Expression::Identifier(name, span.clone())),
                                arguments: Vec::new(),
                                span,
                            }))
                        } else {
                            // Has arguments - this is a struct instantiation with parameters
                            Ok(Expression::StructInstantiation(StructInstantiation {
                                name,
                                arguments,
                                initializer: None,
                                span,
                            }))
                        }
                    }
                } else if self.check(&TokenKind::LeftBrace) {
                    // Direct struct instantiation without parentheses
                    self.advance(); // consume '{'
                    let mut field_assignments = Vec::new();

                    while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                        // Parse field name
                        if let TokenKind::Identifier = self.peek().kind {
                            let field_name = self.advance().value.clone();

                            // Expect '='
                            if !self.match_token(&TokenKind::Assign) {
                                let current_token = self.peek().clone();
                                return Err(StriaError::parser_with_span_and_help(
                                    format!(
                                        "Expected '=' after field name '{}', found '{}'",
                                        field_name, current_token.value
                                    ),
                                    current_token.span,
                                    "struct fields must be assigned with '=' operator",
                                ));
                            }

                            // Parse field value
                            let field_value = self.parse_expression()?;

                            // Create an assignment expression for this field
                            field_assignments
                                .push(Expression::Identifier(field_name, span.clone()));
                            field_assignments.push(field_value);

                            // Handle comma, semicolon, or end - allow newlines as field separators
                            if self.check(&TokenKind::Comma) || self.check(&TokenKind::Semicolon) {
                                self.advance(); // consume ',' or ';'
                            } else if self.check(&TokenKind::RightBrace) {
                                // End of struct, will be handled by outer loop
                            } else {
                                // Allow implicit field separation (newlines act as separators)
                            }
                        } else {
                            let current_token = self.peek().clone();
                            return Err(StriaError::parser_with_span_and_help(
                                format!("Expected field name, found '{}'", current_token.value),
                                current_token.span,
                                "field names must be valid identifiers",
                            ));
                        }
                    }

                    if !self.match_token(&TokenKind::RightBrace) {
                        let current_token = self.peek().clone();
                        return Err(StriaError::parser_with_span(
                            format!(
                                "Expected '}}' after struct fields, found '{}'",
                                current_token.value
                            ),
                            current_token.span,
                        ));
                    }

                    Ok(Expression::StructInstantiation(StructInstantiation {
                        name,
                        arguments: field_assignments,
                        initializer: None,
                        span,
                    }))
                } else {
                    Ok(Expression::Identifier(name, span))
                }
            }

            TokenKind::LeftParen => {
                self.advance(); // consume '('
                let expr = self.parse_expression()?;

                if !self.match_token(&TokenKind::RightParen) {
                    return Err(StriaError::parser(
                        "Expected ')' after expression".to_string(),
                    ));
                }

                Ok(expr)
            }

            TokenKind::LeftBrace => {
                let block = self.parse_block()?;
                Ok(Expression::Block(block))
            }

            _ => Err(StriaError::parser(format!(
                "Unexpected token: {:?}",
                self.peek().kind
            ))),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> &Token {
        if self.is_at_end() {
            &self.eof_token
        } else {
            &self.tokens[self.current]
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
        }
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_tokens(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn parse_schema_directive(&mut self) -> StriaResult<SchemaDirective> {
        let start = self.advance().span.start;

        // Expect the path to be a string literal
        if !self.check(&TokenKind::StringLiteral) {
            return Err(StriaError::ParserError(format!(
                "Expected string literal after '#schema' directive at line {}",
                start
            )));
        }

        let path_token = self.advance();
        let mut path = path_token.value.clone();

        // Remove quotes from the path if present
        if (path.starts_with('"') && path.ends_with('"'))
            || (path.starts_with('\'') && path.ends_with('\''))
        {
            path = path[1..path.len() - 1].to_string();
        }

        let span = crate::lexer::Span {
            start,
            end: path_token.span.end,
            line: path_token.span.line,
            column: path_token.span.column,
        };

        Ok(SchemaDirective { path, span })
    }
}
