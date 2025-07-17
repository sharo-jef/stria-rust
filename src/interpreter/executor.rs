use crate::error::{StriaError, StriaResult};
use crate::interpreter::stdlib::get_builtin_functions;
use crate::interpreter::value::Value;
use crate::parser::ast::*;
use indexmap::IndexMap;
use std::collections::HashMap;

pub struct Executor {
    scopes: Vec<Scope>,
    functions: HashMap<String, FunctionDeclaration>,
    builtins: HashMap<String, fn(&[Value]) -> StriaResult<Value>>,
    structs: HashMap<String, StructDeclaration>,
    return_value: Option<Value>,
    break_flag: bool,
    continue_flag: bool,
}

#[derive(Debug, Clone)]
pub struct Scope {
    variables: HashMap<String, Value>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

impl Executor {
    pub fn new() -> Self {
        let mut builtins = HashMap::new();
        for (name, func) in get_builtin_functions() {
            builtins.insert(name.to_string(), func);
        }

        Self {
            scopes: vec![Scope::new()],
            functions: HashMap::new(),
            builtins,
            structs: HashMap::new(),
            return_value: None,
            break_flag: false,
            continue_flag: false,
        }
    }

    pub fn execute(&mut self, program: &Program) -> StriaResult<()> {
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

        // Second pass: execute each item
        for item in &program.items {
            self.execute_item(item)?;
        }

        Ok(())
    }

    fn execute_item(&mut self, item: &Item) -> StriaResult<()> {
        match item {
            Item::StructDeclaration(_) => {
                // Already handled in first pass
                Ok(())
            }
            Item::FunctionDeclaration(_) => {
                // Already handled in first pass
                Ok(())
            }
            Item::VariableDeclaration(var_decl) => self.execute_variable_declaration(var_decl),
            Item::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(())
            }
            _ => {
                // Skip other items for now
                Ok(())
            }
        }
    }

    fn execute_variable_declaration(&mut self, var_decl: &VariableDeclaration) -> StriaResult<()> {
        let value = if let Some(ref initializer) = var_decl.initializer {
            self.evaluate_expression(initializer)?
        } else {
            Value::Null
        };

        self.declare_variable(&var_decl.name, value)?;
        Ok(())
    }

    fn execute_block(&mut self, block: &Block) -> StriaResult<()> {
        for stmt in &block.statements {
            self.execute_statement(stmt)?;

            // Check for control flow changes
            if self.return_value.is_some() || self.break_flag || self.continue_flag {
                break;
            }
        }
        Ok(())
    }

    fn execute_statement(&mut self, stmt: &Statement) -> StriaResult<()> {
        match stmt {
            Statement::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(())
            }
            Statement::VariableDeclaration(var_decl) => self.execute_variable_declaration(var_decl),
            Statement::Assignment(assignment) => {
                let value = self.evaluate_expression(&assignment.value)?;
                self.assign_value(&assignment.target, value)?;
                Ok(())
            }
            Statement::Return(return_expr) => {
                let value = if let Some(ref expr) = return_expr {
                    self.evaluate_expression(expr)?
                } else {
                    Value::Null
                };

                self.return_value = Some(value);
                Ok(())
            }
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> StriaResult<Value> {
        match expr {
            Expression::Literal(literal) => Ok(self.literal_to_value(&literal.value)),
            Expression::Identifier(name, _) => self.lookup_variable(name),
            Expression::BinaryOp(binary_op) => {
                let left = self.evaluate_expression(&binary_op.left)?;
                let right = self.evaluate_expression(&binary_op.right)?;

                self.evaluate_binary_operation(&binary_op.operator, &left, &right)
            }
            Expression::UnaryOp(unary_op) => {
                let operand = self.evaluate_expression(&unary_op.operand)?;

                match unary_op.operator {
                    UnaryOperator::Not => Ok(operand.logical_not()),
                    UnaryOperator::Minus => operand.negate(),
                    UnaryOperator::Plus => Ok(operand),
                    UnaryOperator::BitNot => operand.bitwise_not(),
                    UnaryOperator::NullAssert => {
                        // Null assertion - return the value if not null, error if null
                        match operand {
                            Value::Null => {
                                Err(StriaError::runtime("Null assertion failed".to_string()))
                            }
                            Value::Optional(Some(val)) => Ok(*val),
                            Value::Optional(None) => {
                                Err(StriaError::runtime("Null assertion failed".to_string()))
                            }
                            _ => Ok(operand),
                        }
                    }
                }
            }
            Expression::Call(call) => self.call_function(call),
            Expression::MemberAccess(member_access) => {
                let object = self.evaluate_expression(&member_access.object)?;

                match &object {
                    Value::Struct(_, fields) => {
                        if let Some(field_value) = fields.get(&member_access.member) {
                            Ok(field_value.clone())
                        } else {
                            Err(StriaError::runtime(format!(
                                "Struct has no field '{}'",
                                member_access.member
                            )))
                        }
                    }
                    _ => Err(StriaError::runtime(format!(
                        "Cannot access member '{}' on non-struct value",
                        member_access.member
                    ))),
                }
            }
            Expression::IndexAccess(index_access) => {
                let object = self.evaluate_expression(&index_access.object)?;
                let index = self.evaluate_expression(&index_access.index)?;

                match (&object, &index) {
                    (Value::List(elements), Value::Integer(i, _)) => {
                        let idx = *i as usize;
                        if idx < elements.len() {
                            Ok(elements[idx].clone())
                        } else {
                            Err(StriaError::runtime(format!(
                                "List index {} out of bounds (length {})",
                                idx,
                                elements.len()
                            )))
                        }
                    }
                    (Value::List(_), _) => Err(StriaError::runtime(
                        "List index must be an integer".to_string(),
                    )),
                    _ => Err(StriaError::runtime(
                        "Cannot index non-list value".to_string(),
                    )),
                }
            }
            Expression::List(elements, _) => {
                let mut values = Vec::new();
                for element_expr in elements {
                    values.push(self.evaluate_expression(element_expr)?);
                }
                Ok(Value::List(values))
            }
            Expression::StructInstantiation(struct_instantiation) => {
                let struct_decl = self
                    .structs
                    .get(&struct_instantiation.name)
                    .ok_or_else(|| {
                        StriaError::runtime(format!(
                            "Undefined struct '{}'",
                            struct_instantiation.name
                        ))
                    })?
                    .clone();

                let mut fields = IndexMap::new();

                // Initialize required properties to null initially
                for property in &struct_decl.properties {
                    if property.is_optional {
                        fields.insert(property.name.clone(), Value::Optional(None));
                    } else {
                        fields.insert(property.name.clone(), Value::Null);
                    }
                }

                // Apply default values if specified
                for property in &struct_decl.properties {
                    if let Some(default_expr) = &property.default_value {
                        let default_value = self.evaluate_expression(default_expr)?;
                        fields.insert(property.name.clone(), default_value);
                    }
                }

                let mut struct_value = Value::Struct(struct_instantiation.name.clone(), fields);

                // Phase 1: Constructor arguments (if any)
                if !struct_instantiation.arguments.is_empty() {
                    // Find appropriate constructor
                    let mut constructor = None;
                    for init_method in &struct_decl.init_methods {
                        if init_method.parameters.len() == struct_instantiation.arguments.len() {
                            constructor = Some(init_method);
                            break;
                        }
                    }

                    if let Some(init_method) = constructor {
                        // Evaluate arguments and bind to parameters
                        for (arg_expr, param) in struct_instantiation
                            .arguments
                            .iter()
                            .zip(init_method.parameters.iter())
                        {
                            let arg_value = self.evaluate_expression(arg_expr)?;
                            if param.is_this {
                                // Handle primary constructor parameter assignment
                                let param_name = param.name.trim_start_matches("this.");
                                if let Value::Struct(_, ref mut fields) = struct_value {
                                    fields.insert(param_name.to_string(), arg_value);
                                }
                            }
                        }
                    }
                }

                // Phase 2: Postfix lambda (initialization block)
                if let Some(initializer) = &struct_instantiation.initializer {
                    // Create a scope for the struct instance
                    self.push_scope();

                    // Bind struct fields to current scope
                    if let Value::Struct(_, ref fields) = struct_value {
                        for (field_name, field_value) in fields {
                            self.declare_variable(field_name, field_value.clone())?;
                        }
                    }

                    // Execute the initializer block
                    self.execute_block(initializer)?;

                    // Update struct fields from scope
                    if let Value::Struct(_, ref mut fields) = struct_value {
                        for (field_name, _) in fields.clone() {
                            if let Ok(new_value) = self.lookup_variable(&field_name) {
                                fields.insert(field_name, new_value);
                            }
                        }
                    }

                    self.pop_scope();
                }

                // Phase 3: Init method execution
                for init_method in &struct_decl.init_methods {
                    if let Some(body) = &init_method.body {
                        self.push_scope();

                        // Bind struct fields to current scope
                        if let Value::Struct(_, ref fields) = struct_value {
                            for (field_name, field_value) in fields {
                                self.declare_variable(field_name, field_value.clone())?;
                            }
                        }

                        // Execute init method body
                        self.execute_block(body)?;

                        // Update struct fields from scope
                        if let Value::Struct(_, ref mut fields) = struct_value {
                            for (field_name, _) in fields.clone() {
                                if let Ok(new_value) = self.lookup_variable(&field_name) {
                                    fields.insert(field_name, new_value);
                                }
                            }
                        }

                        self.pop_scope();
                    }
                }

                // Validate that all required properties have been assigned
                if let Value::Struct(_, ref fields) = struct_value {
                    for property in &struct_decl.properties {
                        if !property.is_optional {
                            if let Some(field_value) = fields.get(&property.name) {
                                if matches!(field_value, Value::Null) {
                                    return Err(StriaError::runtime(format!(
                                        "Required property '{}' not assigned in struct '{}'",
                                        property.name, struct_instantiation.name
                                    )));
                                }
                            }
                        }
                    }
                }

                Ok(struct_value)
            }
            Expression::Block(block) => {
                self.push_scope();
                let mut result = Value::Null;
                for stmt in &block.statements {
                    match stmt {
                        Statement::Expression(expr) => {
                            result = self.evaluate_expression(expr)?;
                        }
                        _ => {
                            self.execute_statement(stmt)?;
                        }
                    }
                    // Check for control flow changes
                    if self.return_value.is_some() || self.break_flag || self.continue_flag {
                        break;
                    }
                }
                self.pop_scope();
                Ok(result)
            }
            Expression::If(if_expr) => {
                let condition = self.evaluate_expression(&if_expr.condition)?;
                if condition.is_truthy() {
                    self.evaluate_expression(&if_expr.then_branch)
                } else if let Some(else_branch) = &if_expr.else_branch {
                    self.evaluate_expression(else_branch)
                } else {
                    Ok(Value::Null)
                }
            }
            Expression::Match(match_expr) => {
                let value = self.evaluate_expression(&match_expr.value)?;

                for arm in &match_expr.arms {
                    let matches = self.pattern_matches(&arm.pattern, &value)?;
                    if matches {
                        return self.evaluate_expression(&arm.body);
                    }
                }

                Err(StriaError::runtime("No matching pattern found".to_string()))
            }
            Expression::Lambda(_lambda) => {
                // For now, store lambda as a function name
                // In a full implementation, we would need to capture the current scope
                Ok(Value::Function("<lambda>".to_string()))
            }
            Expression::Range(range) => {
                let start = self.evaluate_expression(&range.start)?;
                let end = self.evaluate_expression(&range.end)?;

                match (&start, &end) {
                    (Value::Integer(from, _), Value::Integer(to, _)) => {
                        let to_val = if range.inclusive {
                            *to as i32
                        } else {
                            *to as i32 - 1
                        };
                        Ok(Value::I32Range {
                            from: *from as i32,
                            to: to_val,
                            step: 1,
                        })
                    }
                    (Value::Float(from, _), Value::Float(to, _)) => {
                        let to_val = if range.inclusive { *to } else { *to };
                        Ok(Value::F64Range {
                            from: *from,
                            to: to_val,
                            step: 1.0,
                        })
                    }
                    (Value::Integer(from, _), Value::Float(to, _)) => {
                        let to_val = if range.inclusive { *to } else { *to };
                        Ok(Value::F64Range {
                            from: *from as f64,
                            to: to_val,
                            step: 1.0,
                        })
                    }
                    (Value::Float(from, _), Value::Integer(to, _)) => {
                        let to_val = if range.inclusive {
                            *to as f64
                        } else {
                            *to as f64
                        };
                        Ok(Value::F64Range {
                            from: *from,
                            to: to_val,
                            step: 1.0,
                        })
                    }
                    _ => Err(StriaError::runtime(
                        "Range requires numeric operands".to_string(),
                    )),
                }
            }
            Expression::TypeCast(type_cast) => {
                let value = self.evaluate_expression(&type_cast.expression)?;

                // Extract type name from type annotation
                let type_name = match &type_cast.target_type.type_expr {
                    TypeExpression::Primitive(prim) => match prim {
                        PrimitiveType::String => "string",
                        PrimitiveType::Bool => "bool",
                        PrimitiveType::I8 => "i8",
                        PrimitiveType::I16 => "i16",
                        PrimitiveType::I32 => "i32",
                        PrimitiveType::I64 => "i64",
                        PrimitiveType::U8 => "u8",
                        PrimitiveType::U16 => "u16",
                        PrimitiveType::U32 => "u32",
                        PrimitiveType::U64 => "u64",
                        PrimitiveType::F32 => "f32",
                        PrimitiveType::F64 => "f64",
                    },
                    TypeExpression::Identifier(name) => name.as_str(),
                    _ => {
                        return Err(StriaError::runtime(
                            "Complex type casting not yet supported".to_string(),
                        ))
                    }
                };

                value.cast_to(type_name)
            }
            _ => {
                // Skip other expressions for now
                Ok(Value::Null)
            }
        }
    }

    fn literal_to_value(&self, literal: &LiteralValue) -> Value {
        match literal {
            LiteralValue::Integer(i, int_type) => Value::Integer(*i, int_type.clone()),
            LiteralValue::Float(f, float_type) => Value::Float(*f, float_type.clone()),
            LiteralValue::String(s) => Value::String(s.clone()),
            LiteralValue::Boolean(b) => Value::Boolean(*b),
            LiteralValue::Null => Value::Null,
        }
    }

    fn evaluate_binary_operation(
        &self,
        op: &BinaryOperator,
        left: &Value,
        right: &Value,
    ) -> StriaResult<Value> {
        match op {
            BinaryOperator::Add => left.add(right),
            BinaryOperator::Subtract => left.subtract(right),
            BinaryOperator::Multiply => left.multiply(right),
            BinaryOperator::Divide => left.divide(right),
            BinaryOperator::Modulo => left.modulo(right),
            BinaryOperator::Power => left.power(right),
            BinaryOperator::Equal => Ok(Value::Boolean(left.equals(right))),
            BinaryOperator::NotEqual => Ok(Value::Boolean(!left.equals(right))),
            BinaryOperator::Less => {
                let ordering = left.compare(right)?;
                Ok(Value::Boolean(ordering == std::cmp::Ordering::Less))
            }
            BinaryOperator::Greater => {
                let ordering = left.compare(right)?;
                Ok(Value::Boolean(ordering == std::cmp::Ordering::Greater))
            }
            BinaryOperator::LessEqual => {
                let ordering = left.compare(right)?;
                Ok(Value::Boolean(ordering != std::cmp::Ordering::Greater))
            }
            BinaryOperator::GreaterEqual => {
                let ordering = left.compare(right)?;
                Ok(Value::Boolean(ordering != std::cmp::Ordering::Less))
            }
            BinaryOperator::And => Ok(Value::Boolean(left.is_truthy() && right.is_truthy())),
            BinaryOperator::Or => Ok(Value::Boolean(left.is_truthy() || right.is_truthy())),
            BinaryOperator::BitAnd => left.bitwise_and(right),
            BinaryOperator::BitOr => left.bitwise_or(right),
            BinaryOperator::BitXor => left.bitwise_xor(right),
            BinaryOperator::LeftShift => left.left_shift(right),
            BinaryOperator::RightShift => left.right_shift(right),
            BinaryOperator::Range => {
                // Create a range from left to right (exclusive)
                match (left, right) {
                    (Value::Integer(from, _), Value::Integer(to, _)) => Ok(Value::I32Range {
                        from: *from as i32,
                        to: *to as i32 - 1,
                        step: 1,
                    }),
                    (Value::Float(from, _), Value::Float(to, _)) => Ok(Value::F64Range {
                        from: *from,
                        to: *to,
                        step: 1.0,
                    }),
                    (Value::Integer(from, _), Value::Float(to, _)) => Ok(Value::F64Range {
                        from: *from as f64,
                        to: *to,
                        step: 1.0,
                    }),
                    (Value::Float(from, _), Value::Integer(to, _)) => Ok(Value::F64Range {
                        from: *from,
                        to: *to as f64,
                        step: 1.0,
                    }),
                    _ => Err(StriaError::runtime(
                        "Range operator requires numeric operands".to_string(),
                    )),
                }
            }
            BinaryOperator::RangeInclusive => {
                // Create a range from left to right (inclusive)
                match (left, right) {
                    (Value::Integer(from, _), Value::Integer(to, _)) => Ok(Value::I32Range {
                        from: *from as i32,
                        to: *to as i32,
                        step: 1,
                    }),
                    (Value::Float(from, _), Value::Float(to, _)) => Ok(Value::F64Range {
                        from: *from,
                        to: *to,
                        step: 1.0,
                    }),
                    (Value::Integer(from, _), Value::Float(to, _)) => Ok(Value::F64Range {
                        from: *from as f64,
                        to: *to,
                        step: 1.0,
                    }),
                    (Value::Float(from, _), Value::Integer(to, _)) => Ok(Value::F64Range {
                        from: *from,
                        to: *to as f64,
                        step: 1.0,
                    }),
                    _ => Err(StriaError::runtime(
                        "Range operator requires numeric operands".to_string(),
                    )),
                }
            }
            BinaryOperator::In => {
                let contains = right.contains(left)?;
                Ok(Value::Boolean(contains))
            }
            BinaryOperator::Step => left.step(right),
            BinaryOperator::Until => left.until(right),
            BinaryOperator::DownTo => left.down_to(right),
            BinaryOperator::Is => {
                // Type checking - right should be a type name (string)
                if let Value::String(type_name) = right {
                    Ok(Value::Boolean(left.is_type(type_name)))
                } else {
                    Err(StriaError::runtime(
                        "is operator requires a type name".to_string(),
                    ))
                }
            }
            BinaryOperator::As => {
                // Type casting - right should be a type name (string)
                if let Value::String(type_name) = right {
                    left.cast_to(type_name)
                } else {
                    Err(StriaError::runtime(
                        "as operator requires a type name".to_string(),
                    ))
                }
            }
        }
    }

    fn call_function(&mut self, call: &Call) -> StriaResult<Value> {
        // For now, handle simple function calls by name
        if let Expression::Identifier(func_name, _) = &*call.callee {
            // Check if it's a builtin function first
            if self.builtins.contains_key(func_name) {
                let mut args = Vec::new();
                for arg_expr in &call.arguments {
                    args.push(self.evaluate_expression(arg_expr)?);
                }
                let builtin_func = self.builtins.get(func_name).unwrap();
                return builtin_func(&args);
            }

            // Check if it's a user-defined function
            let func_decl = self
                .functions
                .get(func_name)
                .ok_or_else(|| StriaError::runtime(format!("Undefined function '{}'", func_name)))?
                .clone();

            if call.arguments.len() != func_decl.parameters.len() {
                return Err(StriaError::runtime(format!(
                    "Function '{}' expects {} arguments, got {}",
                    func_name,
                    func_decl.parameters.len(),
                    call.arguments.len()
                )));
            }

            // Create new scope for function execution
            self.push_scope();

            // Evaluate arguments and bind to parameters
            for (arg_expr, param) in call.arguments.iter().zip(func_decl.parameters.iter()) {
                let arg_value = self.evaluate_expression(arg_expr)?;
                self.declare_variable(&param.name, arg_value)?;
            }

            // Execute function body
            let mut result = Value::Null;
            self.execute_block(&func_decl.body)?;

            if let Some(return_value) = self.return_value.take() {
                result = return_value;
            }

            self.pop_scope();
            Ok(result)
        } else {
            Err(StriaError::runtime(
                "Complex function calls not yet implemented".to_string(),
            ))
        }
    }

    fn assign_value(&mut self, target: &Expression, value: Value) -> StriaResult<()> {
        match target {
            Expression::Identifier(name, _) => {
                self.set_variable(name, value)?;
                Ok(())
            }
            Expression::MemberAccess(member_access) => {
                let object = self.evaluate_expression(&member_access.object)?;

                match object {
                    Value::Struct(name, mut fields) => {
                        fields.insert(member_access.member.clone(), value);
                        let updated_struct = Value::Struct(name, fields);
                        self.assign_value(&member_access.object, updated_struct)?;
                        Ok(())
                    }
                    _ => Err(StriaError::runtime(
                        "Cannot assign to member of non-struct value".to_string(),
                    )),
                }
            }
            Expression::IndexAccess(index_access) => {
                let array = self.evaluate_expression(&index_access.object)?;
                let index = self.evaluate_expression(&index_access.index)?;

                match (array, &index) {
                    (Value::List(mut elements), Value::Integer(i, _)) => {
                        let idx = *i as usize;
                        if idx < elements.len() {
                            elements[idx] = value;
                            let updated_array = Value::List(elements);
                            self.assign_value(&index_access.object, updated_array)?;
                            Ok(())
                        } else {
                            Err(StriaError::runtime(format!(
                                "List index {} out of bounds (length {})",
                                idx,
                                elements.len()
                            )))
                        }
                    }
                    (Value::List(_), _) => Err(StriaError::runtime(
                        "List index must be an integer".to_string(),
                    )),
                    _ => Err(StriaError::runtime(
                        "Cannot index non-array value".to_string(),
                    )),
                }
            }
            _ => Err(StriaError::runtime("Invalid assignment target".to_string())),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_variable(&mut self, name: &str, value: Value) -> StriaResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.variables.insert(name.to_string(), value);
            Ok(())
        } else {
            Err(StriaError::runtime(
                "No scope available for variable declaration".to_string(),
            ))
        }
    }

    fn lookup_variable(&self, name: &str) -> StriaResult<Value> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.variables.get(name) {
                return Ok(value.clone());
            }
        }

        Err(StriaError::runtime(format!(
            "Undefined variable '{}'",
            name
        )))
    }

    fn set_variable(&mut self, name: &str, value: Value) -> StriaResult<()> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter_mut().rev() {
            if scope.variables.contains_key(name) {
                scope.variables.insert(name.to_string(), value);
                return Ok(());
            }
        }

        // If not found, create in current scope
        self.declare_variable(name, value)
    }

    fn pattern_matches(&mut self, pattern: &Pattern, value: &Value) -> StriaResult<bool> {
        match pattern {
            Pattern::Literal(literal) => {
                let literal_value = self.literal_to_value(&literal.value);
                Ok(literal_value.equals(value))
            }
            Pattern::Identifier(_name) => {
                // For now, identifiers in patterns are treated as wildcards
                // In a full implementation, we would bind the value to the identifier
                Ok(true)
            }
            Pattern::TypeCheck(type_name, _type_expr) => Ok(value.is_type(type_name)),
            Pattern::Range(range) => {
                let start = self.evaluate_expression(&range.start)?;
                let end = self.evaluate_expression(&range.end)?;

                // Create a range value and check if our value is contained in it
                let range_value = match (&start, &end) {
                    (Value::Integer(from, _), Value::Integer(to, _)) => {
                        let to_val = if range.inclusive {
                            *to as i32
                        } else {
                            *to as i32 - 1
                        };
                        Value::I32Range {
                            from: *from as i32,
                            to: to_val,
                            step: 1,
                        }
                    }
                    (Value::Float(from, _), Value::Float(to, _)) => {
                        let to_val = if range.inclusive { *to } else { *to };
                        Value::F64Range {
                            from: *from,
                            to: to_val,
                            step: 1.0,
                        }
                    }
                    _ => {
                        return Err(StriaError::runtime(
                            "Range pattern requires numeric operands".to_string(),
                        ))
                    }
                };

                range_value.contains(value)
            }
            Pattern::Wildcard => Ok(true),
        }
    }
}
