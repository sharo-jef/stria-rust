use crate::error::{StriaError, StriaResult};
use crate::parser::ast::*;
use std::collections::HashMap;

pub struct SemanticAnalyzer {
    scopes: Vec<Scope>,
    functions: HashMap<String, FunctionDeclaration>,
    structs: HashMap<String, StructDeclaration>,
    imported_functions: std::collections::HashSet<String>,
    current_function: Option<String>,
    return_type: Option<Type>,
    schema_declaration: Option<SchemaDeclaration>,
    schema_reference: Option<SchemaDirective>,
    is_schema_file: bool,
    current_file_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone)]
pub struct Scope {
    variables: HashMap<String, Type>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            imported_functions: std::collections::HashSet::new(),
            current_function: None,
            return_type: None,
            schema_declaration: None,
            schema_reference: None,
            is_schema_file: false,
            current_file_path: None,
        }
    }

    /// Get the collected struct definitions
    pub fn get_structs(&self) -> &HashMap<String, StructDeclaration> {
        &self.structs
    }

    pub fn analyze(&mut self, program: &Program, file_path: Option<&str>) -> StriaResult<()> {
        // Set current file path if provided
        if let Some(path) = file_path {
            self.current_file_path = Some(std::path::PathBuf::from(path));
        }

        // First pass: collect all struct and function declarations
        for item in &program.items {
            match item {
                Item::StructDeclaration(struct_decl) => {
                    self.structs
                        .insert(struct_decl.name.clone(), struct_decl.clone());
                }
                Item::FunctionDeclaration(func_decl) => {
                    self.functions
                        .insert(func_decl.name.clone(), func_decl.clone());
                }
                _ => {}
            }
        }

        // Second pass: analyze each item
        for item in &program.items {
            self.analyze_item(item)?;
        }

        // Third pass: validate schema consistency
        self.validate_schema_consistency()?;

        Ok(())
    }

    fn analyze_item(&mut self, item: &Item) -> StriaResult<()> {
        match item {
            Item::StructDeclaration(struct_decl) => self.analyze_struct(struct_decl),
            Item::FunctionDeclaration(func_decl) => self.analyze_function(func_decl),
            Item::VariableDeclaration(var_decl) => self.analyze_variable_declaration(var_decl),
            Item::UseStatement(use_stmt) => self.analyze_use_statement(use_stmt),
            Item::SchemaDeclaration(schema_decl) => self.analyze_schema_declaration(schema_decl),
            Item::SchemaDirective(schema_dir) => self.analyze_schema_directive(schema_dir),
            Item::Expression(expr) => {
                self.analyze_expression(expr)?;
                Ok(())
            }
        }
    }

    fn analyze_struct(&mut self, struct_decl: &StructDeclaration) -> StriaResult<()> {
        let mut field_names = std::collections::HashSet::new();

        for property in &struct_decl.properties {
            if !field_names.insert(&property.name) {
                return Err(StriaError::semantic(format!(
                    "Duplicate field '{}' in struct '{}'",
                    property.name, struct_decl.name
                )));
            }

            if let Some(ref type_annotation) = property.type_annotation {
                self.validate_type_annotation(type_annotation)?;
            }
        }

        Ok(())
    }

    fn analyze_function(&mut self, func_decl: &FunctionDeclaration) -> StriaResult<()> {
        self.current_function = Some(func_decl.name.clone());

        let return_type = if let Some(ref return_type_annotation) = func_decl.return_type {
            self.type_annotation_to_type(return_type_annotation)?
        } else {
            Type::Void
        };

        self.return_type = Some(return_type);

        // Create new scope for function
        self.push_scope();

        // Add parameters to scope
        for param in &func_decl.parameters {
            let param_type = if let Some(ref type_annotation) = param.type_annotation {
                self.type_annotation_to_type(type_annotation)?
            } else {
                return Err(StriaError::semantic(format!(
                    "Parameter '{}' must have type annotation",
                    param.name
                )));
            };

            self.validate_type(&param_type)?;
            self.declare_variable(&param.name, param_type)?;
        }

        // Analyze function body
        self.analyze_block(&func_decl.body)?;

        self.pop_scope();
        self.current_function = None;
        self.return_type = None;

        Ok(())
    }

    fn analyze_variable_declaration(&mut self, var_decl: &VariableDeclaration) -> StriaResult<()> {
        let declared_type = if let Some(ref type_annotation) = var_decl.type_annotation {
            let t = self.type_annotation_to_type(type_annotation)?;
            self.validate_type(&t)?;
            t
        } else {
            // Infer type from initializer
            if let Some(ref init) = var_decl.initializer {
                self.analyze_expression(init)?
            } else {
                return Err(StriaError::semantic(
                    "Variable declaration must have either type annotation or initializer"
                        .to_string(),
                ));
            }
        };

        if let Some(ref initializer) = var_decl.initializer {
            let init_type = self.analyze_expression(initializer)?;
            if !self.is_compatible_type(&declared_type, &init_type) {
                return Err(StriaError::semantic(format!(
                    "Type mismatch in variable '{}': expected {:?}, got {:?}",
                    var_decl.name, declared_type, init_type
                )));
            }
        }

        self.declare_variable(&var_decl.name, declared_type)?;
        Ok(())
    }

    fn analyze_block(&mut self, block: &Block) -> StriaResult<()> {
        for stmt in &block.statements {
            self.analyze_statement(stmt)?;
        }
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> StriaResult<()> {
        match stmt {
            Statement::Expression(expr) => {
                self.analyze_expression(expr)?;
                Ok(())
            }
            Statement::VariableDeclaration(var_decl) => self.analyze_variable_declaration(var_decl),
            Statement::Assignment(assignment) => {
                let target_type = self.analyze_expression(&assignment.target)?;
                let value_type = self.analyze_expression(&assignment.value)?;

                if !self.is_compatible_type(&target_type, &value_type) {
                    return Err(StriaError::semantic(format!(
                        "Type mismatch in assignment: expected {:?}, got {:?}",
                        target_type, value_type
                    )));
                }

                Ok(())
            }
            Statement::Return(return_expr) => {
                let expected_type = self.return_type.clone().unwrap_or(Type::Void);

                if let Some(ref value) = return_expr {
                    let value_type = self.analyze_expression(value)?;
                    if !self.is_compatible_type(&expected_type, &value_type) {
                        return Err(StriaError::semantic(format!(
                            "Return type mismatch: expected {:?}, got {:?}",
                            expected_type, value_type
                        )));
                    }
                } else if expected_type != Type::Void {
                    return Err(StriaError::semantic(format!(
                        "Function must return a value of type {:?}",
                        expected_type
                    )));
                }

                Ok(())
            }
        }
    }

    fn analyze_expression(&mut self, expr: &Expression) -> StriaResult<Type> {
        match expr {
            Expression::Literal(literal) => Ok(self.get_literal_type(&literal.value)),
            Expression::Identifier(name, _) => self.lookup_variable(name),
            Expression::BinaryOp(binary_op) => {
                let left_type = self.analyze_expression(&binary_op.left)?;
                let right_type = self.analyze_expression(&binary_op.right)?;

                self.analyze_binary_operation(&binary_op.operator, &left_type, &right_type)
            }
            Expression::UnaryOp(unary_op) => {
                let operand_type = self.analyze_expression(&unary_op.operand)?;

                match unary_op.operator {
                    UnaryOperator::Not => {
                        if operand_type != Type::Boolean {
                            return Err(StriaError::semantic(format!(
                                "Logical not operator requires boolean operand, got {:?}",
                                operand_type
                            )));
                        }
                        Ok(Type::Boolean)
                    }
                    UnaryOperator::Minus => {
                        if !self.is_numeric_type(&operand_type) {
                            return Err(StriaError::semantic(format!(
                                "Unary minus requires numeric operand, got {:?}",
                                operand_type
                            )));
                        }
                        Ok(operand_type)
                    }
                    UnaryOperator::Plus => {
                        if !self.is_numeric_type(&operand_type) {
                            return Err(StriaError::semantic(format!(
                                "Unary plus requires numeric operand, got {:?}",
                                operand_type
                            )));
                        }
                        Ok(operand_type)
                    }
                    UnaryOperator::BitNot => {
                        if !self.is_integer_type(&operand_type) {
                            return Err(StriaError::semantic(format!(
                                "Bitwise not operator requires integer operand, got {:?}",
                                operand_type
                            )));
                        }
                        Ok(operand_type)
                    }
                    UnaryOperator::NullAssert => {
                        // For now, just return the operand type
                        Ok(operand_type)
                    }
                }
            }
            Expression::Call(call) => {
                // For now, we'll only handle simple function calls where callee is an identifier
                if let Expression::Identifier(func_name, _) = call.callee.as_ref() {
                    // Check if it's a user-defined function
                    if let Some(func_decl) = self.functions.get(func_name) {
                        let func_decl = func_decl.clone();

                        if call.arguments.len() != func_decl.parameters.len() {
                            return Err(StriaError::semantic(format!(
                                "Function '{}' expects {} arguments, got {}",
                                func_name,
                                func_decl.parameters.len(),
                                call.arguments.len()
                            )));
                        }

                        for (arg, param) in call.arguments.iter().zip(func_decl.parameters.iter()) {
                            let arg_type = self.analyze_expression(arg)?;
                            if let Some(ref param_type_annotation) = param.type_annotation {
                                let param_type =
                                    self.type_annotation_to_type(param_type_annotation)?;
                                if !self.is_compatible_type(&param_type, &arg_type) {
                                    return Err(StriaError::semantic(format!(
                                        "Argument type mismatch in function '{}': expected {:?}, got {:?}",
                                        func_name, param_type, arg_type
                                    )));
                                }
                            }
                        }

                        let return_type =
                            if let Some(ref return_type_annotation) = func_decl.return_type {
                                self.type_annotation_to_type(return_type_annotation)?
                            } else {
                                Type::Void
                            };

                        Ok(return_type)
                    }
                    // Check if it's an imported builtin function
                    else if self.imported_functions.contains(func_name) {
                        // For builtin functions, we'll do minimal validation
                        // and assume they return appropriate types
                        for arg in &call.arguments {
                            self.analyze_expression(arg)?;
                        }

                        // Return type depends on the builtin function
                        let return_type = match func_name.as_str() {
                            "print" => Type::Void,
                            "getEnv" => Type::Optional(Box::new(Type::String)),
                            "random" => Type::Float(FloatType::F64),
                            "int" => Type::Integer(IntegerType::I32),
                            "float" => Type::Float(FloatType::F64),
                            "str" => Type::String,
                            "len" => Type::Integer(IntegerType::I32),
                            _ => Type::Void,
                        };

                        Ok(return_type)
                    } else {
                        return Err(StriaError::semantic(format!(
                            "Undefined function '{}'",
                            func_name
                        )));
                    }
                } else {
                    // For complex callees, just return void for now
                    Ok(Type::Void)
                }
            }
            Expression::MemberAccess(member_access) => {
                let object_type = self.analyze_expression(&member_access.object)?;

                match &object_type {
                    Type::Struct(struct_name) => {
                        let struct_decl = self.structs.get(struct_name).ok_or_else(|| {
                            StriaError::semantic(format!("Undefined struct '{}'", struct_name))
                        })?;

                        let property = struct_decl
                            .properties
                            .iter()
                            .find(|p| p.name == member_access.member)
                            .ok_or_else(|| {
                                StriaError::semantic(format!(
                                    "Struct '{}' has no property '{}'",
                                    struct_name, member_access.member
                                ))
                            })?;

                        if let Some(ref type_annotation) = property.type_annotation {
                            self.type_annotation_to_type(type_annotation)
                        } else {
                            Err(StriaError::semantic(format!(
                                "Property '{}' has no type annotation",
                                property.name
                            )))
                        }
                    }
                    _ => Err(StriaError::semantic(format!(
                        "Cannot access member '{}' on non-struct type {:?}",
                        member_access.member, object_type
                    ))),
                }
            }
            Expression::IndexAccess(index_access) => {
                let array_type = self.analyze_expression(&index_access.object)?;
                let index_type = self.analyze_expression(&index_access.index)?;

                if !self.is_integer_type(&index_type) {
                    return Err(StriaError::semantic(format!(
                        "Array index must be integer, got {:?}",
                        index_type
                    )));
                }

                match &array_type {
                    Type::Array(element_type) => Ok((**element_type).clone()),
                    _ => Err(StriaError::semantic(format!(
                        "Cannot index non-array type {:?}",
                        array_type
                    ))),
                }
            }
            Expression::List(elements, _) => {
                if elements.is_empty() {
                    return Err(StriaError::semantic(
                        "Cannot infer type of empty array literal".to_string(),
                    ));
                }

                let first_type = self.analyze_expression(&elements[0])?;

                for element in &elements[1..] {
                    let element_type = self.analyze_expression(element)?;
                    if !self.is_compatible_type(&first_type, &element_type) {
                        return Err(StriaError::semantic(format!(
                            "Array elements must have same type: expected {:?}, got {:?}",
                            first_type, element_type
                        )));
                    }
                }

                Ok(Type::Array(Box::new(first_type)))
            }
            Expression::StructInstantiation(struct_instantiation) => {
                // First check if this is a direct struct instantiation
                if let Some(struct_decl) = self.structs.get(&struct_instantiation.name) {
                    let struct_decl = struct_decl.clone();
                    
                    // Check if arguments are field assignments (pairs of field_name, field_value)
                    if struct_instantiation.arguments.len() % 2 != 0 {
                        return Err(StriaError::semantic(format!(
                            "Struct '{}' field assignments must be in pairs (field_name, field_value)",
                            struct_instantiation.name
                        )));
                    }

                    let field_assignment_count = struct_instantiation.arguments.len() / 2;
                    if field_assignment_count != struct_decl.properties.len() {
                        return Err(StriaError::semantic(format!(
                            "Struct '{}' expects {} fields, got {}",
                            struct_instantiation.name,
                            struct_decl.properties.len(),
                            field_assignment_count
                        )));
                    }

                    // Check field assignments
                    for i in 0..field_assignment_count {
                        let field_name_idx = i * 2;
                        let field_value_idx = i * 2 + 1;

                        // Get the field name
                        let field_name = match &struct_instantiation.arguments[field_name_idx] {
                            Expression::Identifier(name, _) => name.clone(),
                            _ => {
                                return Err(StriaError::semantic(format!(
                                    "Expected field name as identifier in struct '{}' instantiation",
                                    struct_instantiation.name
                                )))
                            }
                        };

                        // Find the corresponding property
                        let property = struct_decl
                            .properties
                            .iter()
                            .find(|p| p.name == field_name)
                            .ok_or_else(|| {
                                StriaError::semantic(format!(
                                    "Struct '{}' does not have field '{}'",
                                    struct_instantiation.name, field_name
                                ))
                            })?;

                        // Check the field value type
                        let field_value = &struct_instantiation.arguments[field_value_idx];
                        let value_type = self.analyze_expression(field_value)?;

                        if let Some(ref type_annotation) = property.type_annotation {
                            let expected_type = self.type_annotation_to_type(type_annotation)?;
                            if !self.is_compatible_type(&expected_type, &value_type) {
                                return Err(StriaError::semantic(format!(
                                    "Field '{}' of struct '{}' expects type '{:?}', got '{:?}'",
                                    field_name, struct_instantiation.name, expected_type, value_type
                                )));
                            }
                        }
                    }

                    Ok(Type::Struct(struct_instantiation.name.clone()))
                } else {
                    // Check if this is a property-based instantiation from imported schema
                    if let Some(_schema_ref) = &self.schema_reference {
                        // This is a configuration file that references a schema
                        // We need to resolve property-based instantiations
                        return self.resolve_property_based_instantiation(struct_instantiation);
                    }
                    
                    // Not found anywhere
                    return Err(StriaError::semantic(format!(
                        "Undefined struct '{}'",
                        struct_instantiation.name
                    )));
                }
            }
            Expression::If(if_expr) => {
                let condition_type = self.analyze_expression(&if_expr.condition)?;
                if condition_type != Type::Boolean {
                    return Err(StriaError::semantic(format!(
                        "If condition must be boolean, got {:?}",
                        condition_type
                    )));
                }

                let then_type = self.analyze_expression(&if_expr.then_branch)?;

                if let Some(ref else_expr) = if_expr.else_branch {
                    let else_type = self.analyze_expression(else_expr)?;
                    if !self.is_compatible_type(&then_type, &else_type) {
                        return Err(StriaError::semantic(format!(
                            "If-else branches must have compatible types: {:?} vs {:?}",
                            then_type, else_type
                        )));
                    }
                }

                Ok(then_type)
            }
            Expression::Block(block) => {
                self.push_scope();
                let mut result_type = Type::Void;
                for stmt in &block.statements {
                    self.analyze_statement(stmt)?;
                    if let Statement::Expression(expr) = stmt {
                        result_type = self.analyze_expression(expr)?;
                    }
                }
                self.pop_scope();
                Ok(result_type)
            }
            Expression::Match(_) => {
                // For now, just return void
                Ok(Type::Void)
            }
            Expression::Lambda(_) => {
                // For now, just return void
                Ok(Type::Void)
            }
            Expression::Range(_) => {
                // For now, just return void
                Ok(Type::Void)
            }
            Expression::TypeCast(type_cast) => {
                // Analyze the expression being cast
                self.analyze_expression(&type_cast.expression)?;

                // Return the target type
                self.type_annotation_to_type(&type_cast.target_type)
            }
            Expression::Error(_, _) => {
                // Error expressions have no type
                Ok(Type::Void)
            }
        }
    }

    fn get_literal_type(&self, literal: &LiteralValue) -> Type {
        match literal {
            LiteralValue::Integer(_, int_type) => Type::Integer(int_type.clone()),
            LiteralValue::Float(_, float_type) => Type::Float(float_type.clone()),
            LiteralValue::String(_) => Type::String,
            LiteralValue::Boolean(_) => Type::Boolean,
            LiteralValue::Null => Type::Null,
        }
    }

    fn type_annotation_to_type(&self, type_annotation: &TypeAnnotation) -> StriaResult<Type> {
        match &type_annotation.type_expr {
            TypeExpression::Primitive(primitive) => Ok(match primitive {
                PrimitiveType::String => Type::String,
                PrimitiveType::Bool => Type::Boolean,
                PrimitiveType::I8 => Type::Integer(IntegerType::I8),
                PrimitiveType::I16 => Type::Integer(IntegerType::I16),
                PrimitiveType::I32 => Type::Integer(IntegerType::I32),
                PrimitiveType::I64 => Type::Integer(IntegerType::I64),
                PrimitiveType::U8 => Type::Integer(IntegerType::U8),
                PrimitiveType::U16 => Type::Integer(IntegerType::U16),
                PrimitiveType::U32 => Type::Integer(IntegerType::U32),
                PrimitiveType::U64 => Type::Integer(IntegerType::U64),
                PrimitiveType::F32 => Type::Float(FloatType::F32),
                PrimitiveType::F64 => Type::Float(FloatType::F64),
            }),
            TypeExpression::List(element_type) => {
                let element_type = self.type_annotation_to_type(&TypeAnnotation {
                    type_expr: (**element_type).clone(),
                    span: type_annotation.span.clone(),
                })?;
                Ok(Type::Array(Box::new(element_type)))
            }
            TypeExpression::Identifier(name) => match name.as_str() {
                "int" => Ok(Type::Integer(IntegerType::I32)),
                "string" => Ok(Type::String),
                "bool" => Ok(Type::Boolean),
                "float" => Ok(Type::Float(FloatType::F64)),
                "i8" => Ok(Type::Integer(IntegerType::I8)),
                "i16" => Ok(Type::Integer(IntegerType::I16)),
                "i32" => Ok(Type::Integer(IntegerType::I32)),
                "i64" => Ok(Type::Integer(IntegerType::I64)),
                "u8" => Ok(Type::Integer(IntegerType::U8)),
                "u16" => Ok(Type::Integer(IntegerType::U16)),
                "u32" => Ok(Type::Integer(IntegerType::U32)),
                "u64" => Ok(Type::Integer(IntegerType::U64)),
                "f32" => Ok(Type::Float(FloatType::F32)),
                "f64" => Ok(Type::Float(FloatType::F64)),
                _ => {
                    if self.structs.contains_key(name) {
                        Ok(Type::Struct(name.clone()))
                    } else {
                        Err(StriaError::semantic(format!("Undefined type '{}'", name)))
                    }
                }
            },
            _ => {
                // Skip other type expressions for now
                Ok(Type::Void)
            }
        }
    }

    fn analyze_binary_operation(
        &self,
        op: &BinaryOperator,
        left: &Type,
        right: &Type,
    ) -> StriaResult<Type> {
        match op {
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo => {
                if !self.is_numeric_type(left) || !self.is_numeric_type(right) {
                    return Err(StriaError::semantic(format!(
                        "Arithmetic operations require numeric operands, got {:?} and {:?}",
                        left, right
                    )));
                }

                // Return the "wider" type
                if matches!(left, Type::Float(_)) || matches!(right, Type::Float(_)) {
                    Ok(Type::Float(FloatType::F64))
                } else {
                    Ok(Type::Integer(IntegerType::I64))
                }
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                if !self.is_compatible_type(left, right) {
                    return Err(StriaError::semantic(format!(
                        "Cannot compare incompatible types {:?} and {:?}",
                        left, right
                    )));
                }
                Ok(Type::Boolean)
            }
            BinaryOperator::Less
            | BinaryOperator::Greater
            | BinaryOperator::LessEqual
            | BinaryOperator::GreaterEqual => {
                if !self.is_numeric_type(left) || !self.is_numeric_type(right) {
                    return Err(StriaError::semantic(format!(
                        "Comparison operations require numeric operands, got {:?} and {:?}",
                        left, right
                    )));
                }
                Ok(Type::Boolean)
            }
            BinaryOperator::And | BinaryOperator::Or => {
                if *left != Type::Boolean || *right != Type::Boolean {
                    return Err(StriaError::semantic(format!(
                        "Logical operations require boolean operands, got {:?} and {:?}",
                        left, right
                    )));
                }
                Ok(Type::Boolean)
            }
            _ => {
                // Skip other operators for now
                Ok(Type::Void)
            }
        }
    }

    fn validate_type_annotation(&self, type_annotation: &TypeAnnotation) -> StriaResult<()> {
        self.type_annotation_to_type(type_annotation)?;
        Ok(())
    }

    fn validate_type(&self, type_annotation: &Type) -> StriaResult<()> {
        match type_annotation {
            Type::Struct(name) => {
                if !self.structs.contains_key(name) {
                    return Err(StriaError::semantic(format!(
                        "Undefined struct type '{}'",
                        name
                    )));
                }
            }
            Type::Array(element_type) => {
                self.validate_type(element_type)?;
            }
            _ => {} // Built-in types are always valid
        }
        Ok(())
    }

    fn is_compatible_type(&self, expected: &Type, actual: &Type) -> bool {
        expected == actual || (self.is_numeric_type(expected) && self.is_numeric_type(actual))
    }

    fn is_numeric_type(&self, type_annotation: &Type) -> bool {
        matches!(type_annotation, Type::Integer(_) | Type::Float(_))
    }

    fn is_integer_type(&self, type_annotation: &Type) -> bool {
        matches!(type_annotation, Type::Integer(_))
    }

    fn declare_variable(&mut self, name: &str, var_type: Type) -> StriaResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.variables.contains_key(name) {
                return Err(StriaError::semantic(format!(
                    "Variable '{}' already declared in this scope",
                    name
                )));
            }
            scope.variables.insert(name.to_string(), var_type);
        }
        Ok(())
    }

    fn lookup_variable(&self, name: &str) -> StriaResult<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(var_type) = scope.variables.get(name) {
                return Ok(var_type.clone());
            }
        }
        Err(StriaError::semantic(format!(
            "Undefined variable '{}'",
            name
        )))
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    #[allow(dead_code)]
    fn has_return_statement(&self, block: &Block) -> bool {
        for stmt in &block.statements {
            if self.statement_has_return(stmt) {
                return true;
            }
        }
        false
    }

    #[allow(dead_code)]
    fn statement_has_return(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::Return(_) => true,
            Statement::Expression(Expression::If(if_expr)) => {
                let then_has_return =
                    if let Expression::Block(then_block) = if_expr.then_branch.as_ref() {
                        self.has_return_statement(then_block)
                    } else {
                        false
                    };
                let else_has_return = if let Some(ref else_branch) = if_expr.else_branch {
                    if let Expression::Block(else_block) = else_branch.as_ref() {
                        self.has_return_statement(else_block)
                    } else {
                        false
                    }
                } else {
                    false
                };
                then_has_return && else_has_return
            }
            _ => false,
        }
    }

    fn analyze_use_statement(&mut self, use_stmt: &UseStatement) -> StriaResult<()> {
        // Validate that the imported functions exist in the standard library
        for function_name in &use_stmt.functions {
            if !self.is_builtin_function(function_name) {
                return Err(StriaError::semantic(format!(
                    "Unknown function '{}' in use statement",
                    function_name
                )));
            }
            // Track imported functions
            self.imported_functions.insert(function_name.clone());
        }
        Ok(())
    }

    fn is_builtin_function(&self, name: &str) -> bool {
        matches!(
            name,
            "print" | "getEnv" | "random" | "int" | "float" | "str" | "len" | "error"
        )
    }

    fn analyze_schema_declaration(&mut self, schema_decl: &SchemaDeclaration) -> StriaResult<()> {
        // Check if this is a schema file
        if self.schema_reference.is_some() {
            return Err(StriaError::semantic(
                "Schema files cannot contain #schema directives".to_string(),
            ));
        }

        if self.schema_declaration.is_some() {
            return Err(StriaError::semantic(
                "Only one schema declaration is allowed per file".to_string(),
            ));
        }

        self.schema_declaration = Some(schema_decl.clone());
        self.is_schema_file = true;

        // Validate that all items in the schema declaration are valid struct names
        for item in &schema_decl.items {
            if !self.structs.contains_key(&item.type_name) {
                return Err(StriaError::parser_with_span_and_help(
                    format!("Schema references undefined struct '{}'", item.type_name),
                    item.span.clone(),
                    format!("Define struct '{}' or check the spelling", item.type_name),
                ));
            }
        }

        Ok(())
    }

    fn analyze_schema_directive(&mut self, schema_dir: &SchemaDirective) -> StriaResult<()> {
        // Check if this is a configuration file
        if self.schema_declaration.is_some() {
            return Err(StriaError::semantic(
                "Configuration files cannot contain schema declarations".to_string(),
            ));
        }

        if self.schema_reference.is_some() {
            return Err(StriaError::semantic(
                "Only one #schema directive is allowed per file".to_string(),
            ));
        }

        self.schema_reference = Some(schema_dir.clone());
        self.is_schema_file = false;

        // Validate that the schema file exists and is valid
        self.validate_schema_file(&schema_dir.path)?;

        // Parse and import the schema file
        self.import_schema_file(&schema_dir.path)?;

        Ok(())
    }

    fn import_schema_file(&mut self, schema_path: &str) -> StriaResult<()> {
        // Resolve the schema path relative to the current file
        let resolved_path = if let Some(current_file) = &self.current_file_path {
            let current_dir = current_file
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            current_dir.join(schema_path)
        } else {
            std::path::PathBuf::from(schema_path)
        };

        // Read the schema file
        let schema_content = std::fs::read_to_string(&resolved_path).map_err(|e| {
            StriaError::semantic(format!(
                "Failed to read schema file '{}': {}",
                resolved_path.display(),
                e
            ))
        })?;

        // Parse the schema file
        let mut tokenizer = crate::lexer::tokenizer::Tokenizer::new();
        let tokens = tokenizer.tokenize(&schema_content).map_err(|e| {
            StriaError::semantic(format!(
                "Failed to tokenize schema file '{}': {}",
                resolved_path.display(),
                e
            ))
        })?;

        let mut parser = crate::parser::Parser::new(tokens);
        let schema_program = parser.parse().map_err(|e| {
            StriaError::semantic(format!(
                "Failed to parse schema file '{}': {}",
                resolved_path.display(),
                e
            ))
        })?;

        // Create a new analyzer for the schema file
        let mut schema_analyzer = SemanticAnalyzer::new();
        schema_analyzer.current_file_path = Some(resolved_path.clone());
        schema_analyzer.is_schema_file = true;

        // Analyze the schema file
        schema_analyzer.analyze(&schema_program, Some(resolved_path.to_str().unwrap()))?;

        // Import struct definitions from the schema
        for (name, struct_def) in schema_analyzer.structs.iter() {
            self.structs.insert(name.clone(), struct_def.clone());
        }

        Ok(())
    }

    fn validate_schema_file(&self, schema_path: &str) -> StriaResult<()> {
        // Resolve the schema path relative to the current file
        let resolved_path = if let Some(current_file) = &self.current_file_path {
            let current_dir = current_file
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            current_dir.join(schema_path)
        } else {
            std::path::PathBuf::from(schema_path)
        };

        // Check if schema file exists
        if !resolved_path.exists() {
            return Err(StriaError::semantic(format!(
                "Schema file '{}' does not exist",
                resolved_path.display()
            )));
        }

        // TODO: Parse and validate the schema file
        // For now, we'll just check that it has a .stria extension
        if !schema_path.ends_with(".stria") {
            return Err(StriaError::semantic(format!(
                "Schema file '{}' must have .stria extension",
                schema_path
            )));
        }

        Ok(())
    }

    fn validate_schema_consistency(&self) -> StriaResult<()> {
        // Check that configuration files have schema references
        if !self.is_schema_file && self.schema_reference.is_none() {
            return Err(StriaError::semantic(
                "Configuration files must reference a schema using #schema directive".to_string(),
            ));
        }

        // Check that schema files don't have configuration data
        if self.is_schema_file {
            // Schema files should only contain:
            // - Schema declarations (schema {...})
            // - Struct declarations (struct {...})
            // - Function declarations (fun {...})
            // - Use statements (use {...})
            // No variable declarations or expressions should be in schema files
            self.validate_schema_file_contents()?;
        }

        Ok(())
    }

    fn validate_schema_file_contents(&self) -> StriaResult<()> {
        // In a real implementation, this would check that the AST only contains
        // allowed constructs for schema files
        // For now, we'll just return OK as this is a demonstration
        Ok(())
    }

    fn resolve_property_based_instantiation(&self, struct_instantiation: &StructInstantiation) -> StriaResult<Type> {
        // TODO: For now, we'll return an error but this should be implemented
        // to resolve property-based instantiations in configuration files
        Err(StriaError::parser_with_span_and_help(
            format!("Property-based instantiation not yet implemented: '{}'", struct_instantiation.name),
            struct_instantiation.span.clone(),
            "Use struct instantiation with explicit type names instead".to_string(),
        ))
    }
}
