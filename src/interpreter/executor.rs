use crate::parser::ast::*;
use crate::interpreter::value::Value;
use crate::error::{StriaResult, StriaError};
use indexmap::IndexMap;
use std::collections::HashMap;

pub struct Executor {
    scopes: Vec<Scope>,
    functions: HashMap<String, FunctionDeclaration>,
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
        Self {
            scopes: vec![Scope::new()],
            functions: HashMap::new(),
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
                    self.structs.insert(struct_decl.name.clone(), struct_decl.clone());
                }
                Item::FunctionDeclaration(func_decl) => {
                    self.functions.insert(func_decl.name.clone(), func_decl.clone());
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
            Item::VariableDeclaration(var_decl) => {
                self.execute_variable_declaration(var_decl)
            }
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
            Statement::VariableDeclaration(var_decl) => {
                self.execute_variable_declaration(var_decl)
            }
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
            Expression::Literal(literal) => {
                Ok(self.literal_to_value(&literal.value))
            }
            Expression::Identifier(name, _) => {
                self.lookup_variable(name)
            }
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
                    UnaryOperator::BitNot => Err(StriaError::runtime("Bitwise NOT not implemented".to_string())),
                    UnaryOperator::NullAssert => Ok(operand),
                }
            }
            Expression::Call(call) => {
                self.call_function(call)
            }
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
                    )))
                }
            }
            Expression::IndexAccess(index_access) => {
                let object = self.evaluate_expression(&index_access.object)?;
                let index = self.evaluate_expression(&index_access.index)?;
                
                match (&object, &index) {
                    (Value::Array(elements), Value::Integer(i, _)) => {
                        let idx = *i as usize;
                        if idx < elements.len() {
                            Ok(elements[idx].clone())
                        } else {
                            Err(StriaError::runtime(format!(
                                "Array index {} out of bounds (length {})",
                                idx, elements.len()
                            )))
                        }
                    }
                    (Value::Array(_), _) => Err(StriaError::runtime(
                        "Array index must be an integer".to_string()
                    )),
                    _ => Err(StriaError::runtime(
                        "Cannot index non-array value".to_string()
                    ))
                }
            }
            Expression::List(elements, _) => {
                let mut values = Vec::new();
                for element_expr in elements {
                    values.push(self.evaluate_expression(element_expr)?);
                }
                Ok(Value::Array(values))
            }
            Expression::StructInstantiation(struct_instantiation) => {
                let fields = IndexMap::new();
                
                // For now, create a simple struct value with just the name
                // The actual initialization would need to be handled based on the AST structure
                Ok(Value::Struct(struct_instantiation.name.clone(), fields))
            }
            Expression::Block(block) => {
                self.push_scope();
                self.execute_block(block)?;
                self.pop_scope();
                Ok(Value::Null)
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

    fn evaluate_binary_operation(&self, op: &BinaryOperator, left: &Value, right: &Value) -> StriaResult<Value> {
        match op {
            BinaryOperator::Add => left.add(right),
            BinaryOperator::Subtract => left.subtract(right),
            BinaryOperator::Multiply => left.multiply(right),
            BinaryOperator::Divide => left.divide(right),
            BinaryOperator::Modulo => left.modulo(right),
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
            _ => Err(StriaError::runtime(format!(
                "Binary operator {:?} not implemented",
                op
            ))),
        }
    }

    fn call_function(&mut self, call: &Call) -> StriaResult<Value> {
        // For now, handle simple function calls by name
        if let Expression::Identifier(func_name, _) = &*call.callee {
            let func_decl = self.functions.get(func_name)
                .ok_or_else(|| StriaError::runtime(format!(
                    "Undefined function '{}'", func_name
                )))?
                .clone();

            if call.arguments.len() != func_decl.parameters.len() {
                return Err(StriaError::runtime(format!(
                    "Function '{}' expects {} arguments, got {}",
                    func_name, func_decl.parameters.len(), call.arguments.len()
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
            Err(StriaError::runtime("Complex function calls not yet implemented".to_string()))
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
                        "Cannot assign to member of non-struct value".to_string()
                    ))
                }
            }
            Expression::IndexAccess(index_access) => {
                let array = self.evaluate_expression(&index_access.object)?;
                let index = self.evaluate_expression(&index_access.index)?;
                
                match (array, &index) {
                    (Value::Array(mut elements), Value::Integer(i, _)) => {
                        let idx = *i as usize;
                        if idx < elements.len() {
                            elements[idx] = value;
                            let updated_array = Value::Array(elements);
                            self.assign_value(&index_access.object, updated_array)?;
                            Ok(())
                        } else {
                            Err(StriaError::runtime(format!(
                                "Array index {} out of bounds (length {})",
                                idx, elements.len()
                            )))
                        }
                    }
                    (Value::Array(_), _) => Err(StriaError::runtime(
                        "Array index must be an integer".to_string()
                    )),
                    _ => Err(StriaError::runtime(
                        "Cannot index non-array value".to_string()
                    ))
                }
            }
            _ => Err(StriaError::runtime(
                "Invalid assignment target".to_string()
            ))
        }
    }

    fn declare_variable(&mut self, name: &str, value: Value) -> StriaResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.variables.insert(name.to_string(), value);
        }
        Ok(())
    }

    fn set_variable(&mut self, name: &str, value: Value) -> StriaResult<()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.variables.contains_key(name) {
                scope.variables.insert(name.to_string(), value);
                return Ok(());
            }
        }
        Err(StriaError::runtime(format!(
            "Undefined variable '{}'", name
        )))
    }

    fn lookup_variable(&self, name: &str) -> StriaResult<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.variables.get(name) {
                return Ok(value.clone());
            }
        }
        Err(StriaError::runtime(format!(
            "Undefined variable '{}'", name
        )))
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}
