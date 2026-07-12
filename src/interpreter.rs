#![allow(warnings)]
use crate::ast::{BlockStatement, Expression, Identifier, Program, Statement};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    store: HashMap<String, Object>,
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_enclosed(outer: Rc<RefCell<Environment>>) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(outer),
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.store.get(name) {
            Some(val) => Some(val.clone()),
            None => match &self.outer {
                Some(outer_env) => outer_env.borrow().get(name),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: String, val: Object) -> Object {
        self.store.insert(name, val.clone());
        val
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
    String(String),
    Error(String),
    ReturnValue(Box<Object>),
    Function {
        parameters: Vec<Identifier>,
        body: BlockStatement,
        env: Rc<RefCell<Environment>>,
    },
    Break,
    Continue,
}

impl Object {
    pub fn object_type(&self) -> &'static str {
        match self {
            Object::Integer(_) => "INTEGER",
            Object::Float(_) => "FLOAT",
            Object::Boolean(_) => "BOOLEAN",
            Object::Null => "NULL",
            Object::String(_) => "STRING",
            Object::Error(_) => "ERROR",
            Object::ReturnValue(_) => "RETURN_VALUE",
            Object::Function { .. } => "FUNCTION",
            Object::Break => "BREAK",
            Object::Continue => "CONTINUE",
        }
    }

    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(val) => val.to_string(),
            Object::Float(val) => val.to_string(),
            Object::Boolean(val) => val.to_string(),
            Object::Null => "null".to_string(),
            Object::String(val) => val.clone(),
            Object::Error(msg) => format!("ERROR: {}", msg),
            Object::ReturnValue(val) => val.inspect(),
            Object::Function {
                parameters, body, ..
            } => {
                let params: Vec<String> = parameters.iter().map(|p| p.value.clone()).collect();
                format!("func({}) {}", params.join(", "), body)
            }
            Object::Break => "break".to_string(),
            Object::Continue => "continue".to_string(),
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Object::Error(_))
    }
}

pub struct Interpreter {
    pub env: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn eval(&mut self, program: &Program) -> Object {
        let mut result = Object::Null;
        for statement in &program.statements {
            result = self.eval_statement(statement);
            match result {
                Object::ReturnValue(val) => return *val,
                Object::Error(_) => return result,
                _ => {}
            }
        }
        result
    }

    pub fn eval_statement(&mut self, stmt: &Statement) -> Object {
        match stmt {
            Statement::Expression { expression, .. } => self.eval_expression(expression),
            Statement::Let { name, value, .. } => {
                let val = self.eval_expression(value);
                if val.is_error() {
                    return val;
                }
                self.env.borrow_mut().set(name.value.clone(), val.clone());
                val
            }
            Statement::Return { return_value, .. } => {
                let val = self.eval_expression(return_value);
                if val.is_error() {
                    return val;
                }
                Object::ReturnValue(Box::new(val))
            }
            Statement::If {
                condition,
                consequence,
                alternative,
                ..
            } => self.eval_if_statement(condition, consequence, alternative.as_ref()),
            Statement::While {
                condition, body, ..
            } => self.eval_while_loop(condition, body),
            Statement::For {
                initializer,
                condition,
                update,
                body,
                ..
            } => self.eval_for_loop(
                initializer.as_deref(),
                condition.as_ref(),
                update.as_deref(),
                body,
            ),
            Statement::Break { .. } => Object::Break,
            Statement::Continue { .. } => Object::Continue,
            Statement::Block(block) => self.eval_block_statement(block),
            // this catches whatever we have added as part of our statement and panics
            _ => unimplemented!(
                "Yeah!, we can't evaluate whatever statement you are trying to evaluate yet"
            ),
        }
    }

    fn eval_expression(&mut self, expr: &Expression) -> Object {
        match expr {
            Expression::Identifier(ident) => self.eval_identifier(ident),
            Expression::Int { value, .. } => Object::Integer(*value),
            Expression::Float { value, .. } => Object::Float(*value),
            Expression::String { value, .. } => Object::String(value.clone()),
            Expression::Bool { value, .. } => Object::Boolean(*value),
            Expression::Prefix {
                operator, right, ..
            } => {
                let right_val = self.eval_expression(right);
                if right_val.is_error() {
                    return right_val;
                }
                self.eval_prefix_expression(operator, right_val)
            }
            Expression::Infix {
                left,
                operator,
                right,
                ..
            } => {
                let left_val = self.eval_expression(left);
                if left_val.is_error() {
                    return left_val;
                }
                let right_val = self.eval_expression(right);
                if right_val.is_error() {
                    return right_val;
                }
                self.eval_infix_expression(operator, left_val, right_val)
            }
            Expression::Function {
                parameters, body, ..
            } => Object::Function {
                parameters: parameters.clone(),
                body: body.clone(),
                env: Rc::clone(&self.env),
            },
            Expression::Call {
                function,
                arguments,
                ..
            } => {
                let func_val = self.eval_expression(function);
                if func_val.is_error() {
                    return func_val;
                }
                let args_vals = self.eval_expressions(arguments);
                if args_vals.len() == 1 && args_vals[0].is_error() {
                    return args_vals[0].clone();
                }
                self.apply_function(func_val, args_vals)
            }
        }
    }

    fn eval_identifier(&self, ident: &Identifier) -> Object {
        match self.env.borrow().get(&ident.value) {
            Some(val) => val,
            None => Object::Error(format!("identifier not found: {}", ident.value)),
        }
    }

    fn eval_prefix_expression(&self, operator: &str, right: Object) -> Object {
        match operator {
            "!" => self.eval_bang_operator_expression(right),
            "-" => self.eval_minus_prefix_operator_expression(right),
            _ => Object::Error(format!(
                "unknown operator: {}{}",
                operator,
                right.object_type()
            )),
        }
    }

    fn eval_bang_operator_expression(&self, right: Object) -> Object {
        match right {
            Object::Boolean(true) => Object::Boolean(false),
            Object::Boolean(false) => Object::Boolean(true),
            Object::Null => Object::Boolean(true),
            _ => Object::Boolean(false),
        }
    }

    fn eval_minus_prefix_operator_expression(&self, right: Object) -> Object {
        match right {
            Object::Integer(value) => Object::Integer(-value),
            Object::Float(value) => Object::Float(-value),
            _ => Object::Error(format!("unknown operator: -{}", right.object_type())),
        }
    }

    fn eval_infix_expression(&self, operator: &str, left: Object, right: Object) -> Object {
        match (&left, &right) {
            (Object::Integer(l_val), Object::Integer(r_val)) => {
                self.eval_integer_infix_expression(operator, *l_val, *r_val)
            }
            (Object::Float(l_val), Object::Float(r_val)) => {
                self.eval_float_infix_expression(operator, *l_val, *r_val)
            }
            (Object::String(l_val), Object::String(r_val)) => {
                self.eval_string_infix_expression(operator, l_val, r_val)
            }
            _ => {
                if operator == "==" {
                    Object::Boolean(left == right)
                } else if operator == "!=" {
                    Object::Boolean(left != right)
                } else if left.object_type() != right.object_type() {
                    Object::Error(format!(
                        "type mismatch: {} {} {}",
                        left.object_type(),
                        operator,
                        right.object_type()
                    ))
                } else {
                    Object::Error(format!(
                        "unknown operator: {} {} {}",
                        left.object_type(),
                        operator,
                        right.object_type()
                    ))
                }
            }
        }
    }

    fn eval_integer_infix_expression(&self, operator: &str, left: i64, right: i64) -> Object {
        match operator {
            "+" => Object::Integer(left + right),
            "-" => Object::Integer(left - right),
            "*" => Object::Integer(left * right),
            "/" => {
                if right == 0 {
                    Object::Error("division by zero".to_string())
                } else {
                    Object::Integer(left / right)
                }
            }
            "<" => Object::Boolean(left < right),
            ">" => Object::Boolean(left > right),
            "==" => Object::Boolean(left == right),
            "!=" => Object::Boolean(left != right),
            _ => Object::Error(format!("unknown operator: INTEGER {} INTEGER", operator)),
        }
    }

    fn eval_float_infix_expression(&self, operator: &str, left: f64, right: f64) -> Object {
        match operator {
            "+" => Object::Float(left + right),
            "-" => Object::Float(left - right),
            "*" => Object::Float(left * right),
            "/" => {
                if right == 0.0 {
                    Object::Error("division by zero".to_string())
                } else {
                    Object::Float(left / right)
                }
            }
            "<" => Object::Boolean(left < right),
            ">" => Object::Boolean(left > right),
            "==" => Object::Boolean(left == right),
            "!=" => Object::Boolean(left != right),
            _ => Object::Error(format!("unknown operator: FLOAT {} FLOAT", operator)),
        }
    }

    fn eval_string_infix_expression(&self, operator: &str, left: &str, right: &str) -> Object {
        match operator {
            "+" => Object::String(format!("{}{}", left, right)),
            "==" => Object::Boolean(left == right),
            "!=" => Object::Boolean(left != right),
            _ => Object::Error(format!("unknown operator: STRING {} STRING", operator)),
        }
    }

    fn eval_if_statement(
        &mut self,
        condition: &Expression,
        consequence: &BlockStatement,
        alternative: Option<&BlockStatement>,
    ) -> Object {
        let cond_val = self.eval_expression(condition);
        if cond_val.is_error() {
            return cond_val;
        }

        if is_truthy(&cond_val) {
            self.eval_block_statement(consequence)
        } else if let Some(alt) = alternative {
            self.eval_block_statement(alt)
        } else {
            Object::Null
        }
    }

    fn eval_while_loop(&mut self, condition: &Expression, body: &BlockStatement) -> Object {
        let mut result = Object::Null;

        loop {
            let cond_val = self.eval_expression(condition);
            if cond_val.is_error() {
                return cond_val;
            }

            if !is_truthy(&cond_val) {
                break;
            }

            result = self.eval_block_statement(body);
            if result.is_error() {
                return result;
            }

            match result {
                Object::Break => break,
                Object::Continue => continue,
                _ => {}
            }
        }

        result
    }

    fn eval_for_loop(
        &mut self,
        initializer: Option<&Statement>,
        condition: Option<&Expression>,
        update: Option<&Statement>,
        body: &BlockStatement,
    ) -> Object {
        let mut result = Object::Null;

        if let Some(init) = initializer {
            let init_res = self.eval_statement(init);
            if init_res.is_error() {
                return init_res;
            }
        }

        loop {
            if let Some(cond) = condition {
                let cond_val = self.eval_expression(cond);
                if cond_val.is_error() {
                    return cond_val;
                }
                if !is_truthy(&cond_val) {
                    break;
                }
            }

            result = self.eval_block_statement(body);
            if result.is_error() {
                return result;
            }

            if let Object::Break = result {
                break;
            }

            let is_continue = matches!(result, Object::Continue);

            if let Some(upd) = update {
                let upd_res = self.eval_statement(upd);
                if upd_res.is_error() {
                    return upd_res;
                }
            }

            if is_continue {
                continue;
            }
        }

        result
    }

    fn eval_block_statement(&mut self, block: &BlockStatement) -> Object {
        let mut result = Object::Null;

        let env = Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(
            &self.env,
        ))));
        let old_env = std::mem::replace(&mut self.env, env);

        for statement in &block.statements {
            result = self.eval_statement(statement);
            if result.is_error() {
                self.env = old_env;
                return result;
            }

            match result {
                Object::ReturnValue(_) | Object::Break | Object::Continue => {
                    self.env = old_env;
                    return result;
                }
                _ => {}
            }
        }

        self.env = old_env;
        result
    }

    fn eval_expressions(&mut self, exps: &[Expression]) -> Vec<Object> {
        let mut result = Vec::new();
        for e in exps {
            let evaluated = self.eval_expression(e);
            if evaluated.is_error() {
                return vec![evaluated];
            }
            result.push(evaluated);
        }
        result
    }

    fn apply_function(&mut self, fn_obj: Object, args: Vec<Object>) -> Object {
        if let Object::Function {
            parameters,
            body,
            env,
        } = fn_obj
        {
            if parameters.len() != args.len() {
                return Object::Error(format!(
                    "wrong number of arguments. got={}, want={}",
                    args.len(),
                    parameters.len()
                ));
            }

            let new_env = Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(&env))));
            for (i, param) in parameters.iter().enumerate() {
                new_env
                    .borrow_mut()
                    .set(param.value.clone(), args[i].clone());
            }

            let old_env = std::mem::replace(&mut self.env, new_env);
            let result = self.eval_block_statement(&body);
            self.env = old_env;

            match result {
                Object::ReturnValue(val) => *val,
                _ => result,
            }
        } else {
            Object::Error(format!("not a function: {}", fn_obj.object_type()))
        }
    }
}

fn is_truthy(obj: &Object) -> bool {
    match obj {
        Object::Null => false,
        Object::Boolean(val) => *val,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Config;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn test_eval(input: &str) -> Object {
        let mut l = Lexer::new(input, Config::default());
        let mut p = Parser::new(&mut l);
        let program = p.parse_program();
        let errors = p.errors();
        if !errors.is_empty() {
            panic!("Parser errors: {}", errors.join("\n"));
        }
        let mut interpreter = Interpreter::new();
        interpreter.eval(&program)
    }

    fn test_integer_object(obj: &Object, expected: i64) {
        match obj {
            Object::Integer(value) => assert_eq!(*value, expected),
            other => panic!("expected Integer, got {:?}", other),
        }
    }

    fn test_boolean_object(obj: &Object, expected: bool) {
        match obj {
            Object::Boolean(value) => assert_eq!(*value, expected),
            other => panic!("expected Boolean, got {:?}", other),
        }
    }

    fn test_null_object(obj: &Object) {
        match obj {
            Object::Null => {}
            other => panic!("expected Null, got {:?}", other),
        }
    }

    #[test]
    fn test_integer_evaluation() {
        let tests = vec![
            ("5", 5),
            ("10", 10),
            ("-5", -5),
            ("-10", -10),
            ("0", 0),
            ("123456789", 123456789),
            ("0x2A", 42),
            ("0b101010", 42),
            ("-0x2A", -42),
            ("-0b1010", -10),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(&evaluated, expected);
        }
    }

    #[test]
    fn test_integer_infix_expressions() {
        let tests = vec![
            ("5 + 5 + 5 + 5 - 10", 10),
            ("2 * 2 * 2 * 2 * 2", 32),
            ("-50 + 100 + -50", 0),
            ("5 * 2 + 10", 20),
            ("5 + 2 * 10", 25),
            ("20 + 2 * -10", 0),
            ("50 / 2 * 2 + 10", 60),
            ("2 * (5 + 10)", 30),
            ("3 * 3 * 3 + 10", 37),
            ("3 * (3 * 3) + 10", 37),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(&evaluated, expected);
        }
    }

    #[test]
    fn test_integer_comparison_expressions() {
        let tests = vec![
            ("5 < 5", false),
            ("5 == 5", true),
            ("5 != 5", false),
            ("5 > 5", false),
            ("5 < 6", true),
            ("5 > 4", true),
            ("5 == 6", false),
            ("5 != 6", true),
            ("(5 > 4) == true", true),
            ("(5 < 4) == false", true),
            ("(5 == 5) == true", true),
            ("(5 != 5) == false", true),
            ("(5 < 5) == false", true),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_boolean_object(&evaluated, expected);
        }
    }

    #[test]
    fn test_integer_prefix_expressions() {
        let tests = vec![
            ("-5", -5),
            ("-10", -10),
            ("--5", 5),
            ("---5", -5),
            ("-0x2A", -42),
            ("-0b1010", -10),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(&evaluated, expected);
        }
    }

    #[test]
    fn test_integer_error_handling() {
        let tests = vec![
            ("5 + true", "type mismatch: INTEGER + BOOLEAN"),
            ("5 + true; 5", "type mismatch: INTEGER + BOOLEAN"),
            ("-true", "unknown operator: -BOOLEAN"),
            ("true + false", "unknown operator: BOOLEAN + BOOLEAN"),
            ("5; true + false; 5", "unknown operator: BOOLEAN + BOOLEAN"),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            match evaluated {
                Object::Error(msg) => assert_eq!(msg, expected),
                other => panic!("expected Error, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_integer_variable_assignments() {
        let tests = vec![
            ("const a = 5; a", 5),
            ("const a = 5 * 5; a", 25),
            ("const a = 5; const b = a; b", 5),
            ("const a = 5; const b = a; const c = a + b + 5; c", 15),
            ("const a = 0x2A; a", 42),
            ("const a = 0b1010; a", 10),
            ("const a = -0x2A; a", -42),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(&evaluated, expected);
        }
    }

    #[test]
    fn test_if_expressions() {
        let tests = vec![
            ("if (true) { 10 }", Some(10)),
            ("if (false) { 10 }", None),
            ("if (1) { 10 }", Some(10)),
            ("if (1 < 2) { 10 }", Some(10)),
            ("if (1 > 2) { 10 } else { 20 }", Some(20)),
            ("if (1 < 2) { 10 } else { 20 }", Some(10)),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            if let Some(val) = expected {
                test_integer_object(&evaluated, val);
            } else {
                test_null_object(&evaluated);
            }
        }
    }

    #[test]
    fn test_while_loops() {
        let tests = vec![
            ("const x = 0; while (false) { const x = 1; } x", 0),
            ("while (false) { const x = 1; } 5", 5),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(&evaluated, expected);
        }
    }

    // This is a stub
    #[test]
    fn test_for_loops() {
        let tests = vec![(
            "const x = 0; for (const i = 0; i < 5; const i = i + 1) { const x = x + i; } 5;",
            5,
        )];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(&evaluated, expected);
        }
    }

    //This is a stub!!!
    #[test]
    fn test_break_and_continue() {
        let tests = vec![
            ("const x = 0; while (true) { break; } x", 0),
            ("while (true) { break; } 5", 5),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(&evaluated, expected);
        }
    }

    #[test]
    fn test_block_statements() {
        let tests = vec![
            ("const x = 10; { const y = 20; y } x", 10),
            ("const x = 10; { const y = 20; const z = x + y; z } x", 10),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(&evaluated, expected);
        }
    }

    #[test]
    fn test_nested_control_structures() {
        let tests = vec![
            (
                "const x = 0; if (true) { if (true) { const x = 10; } } x",
                0,
            ),
            ("if (true) { if (true) { 10; } }", 10),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(&evaluated, expected);
        }
    }

    #[test]
    fn test_function_literals() {
        let input = "const fn = func(x) { x + 2; }; fn";
        let evaluated = test_eval(input);
        match evaluated {
            Object::Function {
                parameters, body, ..
            } => {
                assert_eq!(parameters.len(), 1);
                assert_eq!(parameters[0].value, "x");
                assert_eq!(body.to_string(), "(x + 2)");
            }
            other => panic!("expected Function object, got {:?}", other),
        }
    }

    #[test]
    fn test_function_application() {
        let tests = vec![
            ("const identity = func(x) { x; }; identity(5);", 5),
            ("const double = func(x) { x * 2; }; double(5);", 10),
            ("const add = func(x, y) { x + y; }; add(5, 5);", 10),
            (
                "const add = func(x, y) { x + y; }; add(5 + 5, add(5, 5));",
                20,
            ),
            ("const fn = func(x) { x; }; fn(5);", 5),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(&evaluated, expected);
        }
    }

    #[test]
    fn test_closures() {
        let input = "
        const newAdder = func(x) {
            return func(y) { x + y; };
        };
        const addTwo = newAdder(2);
        addTwo(2);
        ";
        let evaluated = test_eval(input);
        test_integer_object(&evaluated, 4);
    }

    #[test]
    fn test_recursive_functions() {
        let input = "
        const countDown = func(x) {
            if (x > 0) {
                return countDown(x - 1);
            } else {
                return x;
            }
        };
        countDown(1);
        ";
        let evaluated = test_eval(input);
        test_integer_object(&evaluated, 0);
    }

    #[test]
    fn test_return_statements() {
        let tests = vec![
            ("return 10;", 10),
            ("return 10; 9;", 10),
            ("return 2 * 5; 9;", 10),
            ("9; return 2 * 5; 9;", 10),
            ("if (10 > 1) { if (10 > 1) { return 10; } return 1; }", 10),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(&evaluated, expected);
        }
    }

    #[test]
    fn test_function_scope() {
        let input = "
        const x = 10;
        const add = func(y) {
            x + y;
        };
        add(5);
        ";
        let evaluated = test_eval(input);
        test_integer_object(&evaluated, 15);
    }

    #[test]
    fn test_function_error_handling() {
        let tests = vec![
            ("5(5);", "not a function: INTEGER"),
            (
                "const add = func(x, y) { x + y; }; add(5);",
                "wrong number of arguments. got=1, want=2",
            ),
            (
                "const add = func(x, y) { x + y; }; add(5, 5, 5);",
                "wrong number of arguments. got=3, want=2",
            ),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            match evaluated {
                Object::Error(msg) => assert_eq!(msg, expected),
                other => panic!("expected Error, got {:?}", other),
            }
        }
    }
}
