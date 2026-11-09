use crate::environment::Environment;
use crate::error::RuntimeError;
use crate::evaluator::evaluate_expression_with_callbacks;
use crate::parser::{Rule, parse_program};
use crate::runtime::Value;
use pest::iterators::Pair;
use std::cell::RefCell;

#[derive(Debug)]
pub enum LarpError {
    Syntax(String),
    Runtime(RuntimeError),
}

impl std::fmt::Display for LarpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax(message) => write!(f, "Syntax error: {message}"),
            Self::Runtime(error) => write!(f, "Runtime error: {error}"),
        }
    }
}

#[derive(Debug)]
enum Flow {
    Normal,
    Break,
    Continue,
    Return(Value),
}

pub struct Interpreter {
    environment: RefCell<Environment>,
    output: RefCell<Vec<String>>,
    loop_depth: RefCell<usize>,
    function_depth: RefCell<usize>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: RefCell::new(Environment::new()),
            output: RefCell::new(Vec::new()),
            loop_depth: RefCell::new(0),
            function_depth: RefCell::new(0),
        }
    }
    pub fn run_source(&mut self, source: &str) -> Result<(), LarpError> {
        self.execute_source(source).map_err(|error| match error {
            ExecutionError::Runtime(error) => LarpError::Runtime(error),
            ExecutionError::Syntax(error) => LarpError::Syntax(error),
            ExecutionError::Control => LarpError::Runtime(RuntimeError::MalformedStatement),
        })
    }
    pub fn take_output(&mut self) -> Vec<String> {
        std::mem::take(&mut *self.output.borrow_mut())
    }
    fn execute_source(&self, source: &str) -> Result<(), ExecutionError> {
        let mut pairs =
            parse_program(source).map_err(|error| ExecutionError::Syntax(error.to_string()))?;
        let program = pairs
            .next()
            .ok_or_else(|| ExecutionError::Syntax("Empty program".into()))?;
        match self
            .eval_program(program)
            .map_err(ExecutionError::Runtime)?
        {
            Flow::Normal => Ok(()),
            Flow::Break | Flow::Continue | Flow::Return(_) => Err(ExecutionError::Control),
        }
    }

    fn eval_program(&self, program: Pair<Rule>) -> Result<Flow, RuntimeError> {
        for statement in program.into_inner() {
            if statement.as_rule() == Rule::EOI {
                continue;
            }
            let flow = self.eval_statement(statement)?;
            if !matches!(flow, Flow::Normal) {
                return Ok(flow);
            }
        }
        Ok(Flow::Normal)
    }
    fn eval_statement(&self, statement: Pair<Rule>) -> Result<Flow, RuntimeError> {
        match statement.as_rule() {
            Rule::variable_decl => {
                self.eval_declaration(statement, false)?;
            }
            Rule::constant_decl => {
                self.eval_declaration(statement, true)?;
            }
            Rule::assignment => {
                self.eval_assignment(statement)?;
            }
            Rule::print_stmt => {
                self.eval_print(statement)?;
            }
            Rule::if_stmt => {
                return self.eval_if(statement);
            }
            Rule::loop_stmt => {
                return self.eval_loop(statement);
            }
            Rule::grind_stmt => {
                return self.eval_grind(statement);
            }
            Rule::function_decl => {
                self.eval_function_declaration(statement)?;
            }
            Rule::break_stmt => {
                if *self.loop_depth.borrow() == 0 {
                    return Err(RuntimeError::BreakOutsideLoop);
                }
                return Ok(Flow::Break);
            }
            Rule::continue_stmt => {
                if *self.loop_depth.borrow() == 0 {
                    return Err(RuntimeError::ContinueOutsideLoop);
                }
                return Ok(Flow::Continue);
            }
            Rule::expression_stmt => {
                let expression = statement
                    .into_inner()
                    .next()
                    .ok_or(RuntimeError::MalformedStatement)?;

                self.eval_expression(expression)?;
            }
            Rule::return_stmt => {
                if *self.function_depth.borrow() == 0 {
                    return Err(RuntimeError::ReturnOutsideFunction);
                }
                let expression = statement.into_inner().next();
                let value = match expression {
                    Some(expr) => self.eval_expression(expr)?,
                    None => Value::Null,
                };
                return Ok(Flow::Return(value));
            }
            rule => return Err(RuntimeError::UnexpectedToken(rule)),
        }
        Ok(Flow::Normal)
    }
    fn eval_declaration(
        &self,
        statement: Pair<Rule>,
        is_constant: bool,
    ) -> Result<(), RuntimeError> {
        let mut inner = statement.into_inner();
        let name = inner
            .next()
            .ok_or(RuntimeError::MalformedStatement)?
            .as_str()
            .to_string();
        let value = match inner.next() {
            Some(expr) => self.eval_expression(expr)?,
            None => Value::Null,
        };
        self.environment
            .borrow_mut()
            .define_variable(name, value, is_constant)
    }
    fn eval_assignment(&self, statement: Pair<Rule>) -> Result<(), RuntimeError> {
        let mut inner = statement.into_inner();
        let name = inner
            .next()
            .ok_or(RuntimeError::MalformedStatement)?
            .as_str()
            .to_string();
        let expression = inner.next().ok_or(RuntimeError::MalformedStatement)?;
        let value = self.eval_expression(expression)?;
        self.environment.borrow_mut().assign_variable(&name, value)
    }
    fn eval_print(&self, statement: Pair<Rule>) -> Result<(), RuntimeError> {
        let expression = statement
            .into_inner()
            .next()
            .ok_or(RuntimeError::MalformedStatement)?;
        let value = self.eval_expression(expression)?;
        self.output.borrow_mut().push(value.to_string());
        Ok(())
    }
    fn eval_if(&self, statement: Pair<Rule>) -> Result<Flow, RuntimeError> {
    let mut inner = statement.into_inner();
    let condition = inner
        .next()
        .ok_or(RuntimeError::MalformedStatement)?;
    let then_branch = inner
        .next()
        .ok_or(RuntimeError::MalformedStatement)?;
    let else_branch = inner.next();
    match self.eval_expression(condition)? {
        Value::Boolean(true) => {
            self.eval_block(then_branch)
        }
        Value::Boolean(false) => {
            match else_branch {
                Some(branch) if branch.as_rule() == Rule::if_stmt => {
                    self.eval_if(branch)
                }
                Some(branch) => {
                    self.eval_block(branch)
                }
                None => {
                    Ok(Flow::Normal)
                }
            }
        }
        _ => Err(RuntimeError::InvalidCondition),
    }
}
    fn eval_block(&self, block: Pair<Rule>) -> Result<Flow, RuntimeError> {
        self.environment.borrow_mut().push_scope();
        let mut result = Flow::Normal;
        for statement in block.into_inner() {
            let flow = self.eval_statement(statement)?;
            if !matches!(flow, Flow::Normal) {
                result = flow;
                break;
            }
        }
        self.environment.borrow_mut().pop_scope();
        Ok(result)
    }
    fn eval_loop(&self, statement: Pair<Rule>) -> Result<Flow, RuntimeError> {
        let mut inner = statement.into_inner();
        let condition = inner.next().ok_or(RuntimeError::MalformedStatement)?;
        let body = inner.next().ok_or(RuntimeError::MalformedStatement)?;
        *self.loop_depth.borrow_mut() += 1;
        let result = loop {
            let condition_value = self.eval_expression(condition.clone())?;
            match condition_value {
                Value::Boolean(false) => break Ok(Flow::Normal),
                Value::Boolean(true) => {}
                _ => break Err(RuntimeError::InvalidCondition),
            }
            match self.eval_block(body.clone())? {
                Flow::Normal | Flow::Continue => continue,
                Flow::Break => break Ok(Flow::Normal),
                Flow::Return(value) => break Ok(Flow::Return(value)),
            }
        };
        *self.loop_depth.borrow_mut() -= 1;
        result
    }
    fn eval_grind(&self, statement: Pair<Rule>) -> Result<Flow, RuntimeError> {
        let mut inner = statement.into_inner();
        let name = inner
            .next()
            .ok_or(RuntimeError::MalformedStatement)?
            .as_str()
            .to_string();
        let iterable = inner.next().ok_or(RuntimeError::MalformedStatement)?;
        let body = inner.next().ok_or(RuntimeError::MalformedStatement)?;
        let items: Vec<Value> = match self.eval_expression(iterable)? {
            Value::Number(n) if n >= 0.0 && n.fract() == 0.0 => {
                (0..n as u64).map(|k| Value::Number(k as f64)).collect()
            }
            Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
            _ => return Err(RuntimeError::InvalidIterable),
        };
        *self.loop_depth.borrow_mut() += 1;
        let mut result = Ok(Flow::Normal);
        for item in items {
            self.environment.borrow_mut().push_scope();
            // Two statements on purpose: the environment borrow must end before eval_block
            // borrows it again, or the RefCell panics with "already borrowed".
            let defined = self
                .environment
                .borrow_mut()
                .define_variable(name.clone(), item, false);
            let step = defined.and_then(|_| self.eval_block(body.clone()));
            self.environment.borrow_mut().pop_scope();
            match step {
                Ok(Flow::Normal | Flow::Continue) => {}
                Ok(Flow::Break) => break,
                Ok(Flow::Return(value)) => {
                    result = Ok(Flow::Return(value));
                    break;
                }
                Err(error) => {
                    result = Err(error);
                    break;
                }
            }
        }
        *self.loop_depth.borrow_mut() -= 1;
        result
    }
    fn eval_function_declaration(&self, statement: Pair<Rule>) -> Result<(), RuntimeError> {
        let mut inner = statement.clone().into_inner();
        let name = inner
            .next()
            .ok_or(RuntimeError::MalformedStatement)?
            .as_str()
            .to_string();
        let mut params = Vec::new();
        let mut body = None;
        for item in inner {
            if item.as_rule() == Rule::block {
                body = Some(item.as_str().to_string());
                break;
            }
            params.push(item.as_str().to_string());
        }
        let body = body.ok_or(RuntimeError::MalformedStatement)?;
        let function = Value::Function {
            name: name.clone(),
            params,
            body,
        };
        self.environment
            .borrow_mut()
            .define_variable(name, function, true)
    }
    fn call_function(&self, name: &str, arguments: Vec<Value>) -> Result<Value, RuntimeError> {
        let function = self.environment.borrow().get_variable(name)?;
        let (params, body) = match function {
            Value::Function { params, body, .. } => (params, body),
            _ => return Err(RuntimeError::UndefinedFunction(name.to_string())),
        };
        if params.len() != arguments.len() {
            return Err(RuntimeError::ArgumentCountMismatch {
                name: name.to_string(),
                expected: params.len(),
                received: arguments.len(),
            });
        }
        self.environment.borrow_mut().push_scope();
        for (param, argument) in params.into_iter().zip(arguments) {
            self.environment
                .borrow_mut()
                .define_variable(param, argument, false)?;
        }
        *self.function_depth.borrow_mut() += 1;
        let result = self.execute_function_body(&body);
        *self.function_depth.borrow_mut() -= 1;
        self.environment.borrow_mut().pop_scope();
        result
    }
    fn execute_function_body(&self, body: &str) -> Result<Value, RuntimeError> {
        let content = body
            .strip_prefix('{')
            .and_then(|s| s.strip_suffix('}'))
            .ok_or(RuntimeError::MalformedStatement)?;
        let mut pairs = parse_program(content).map_err(|_| RuntimeError::MalformedStatement)?;
        let program = pairs.next().ok_or(RuntimeError::MalformedStatement)?;
        match self.eval_program(program)? {
            Flow::Return(value) => Ok(value),
            Flow::Normal => Ok(Value::Null),
            Flow::Break | Flow::Continue => Err(RuntimeError::MalformedStatement),
        }
    }

    fn eval_expression(&self, expression: Pair<Rule>) -> Result<Value, RuntimeError> {
        let mut lookup = |name: &str| self.environment.borrow().get_variable(name);
        let mut call = |name: &str, args: Vec<Value>| self.call_function(name, args);
        evaluate_expression_with_callbacks(expression, &mut lookup, &mut call)
    }
}

enum ExecutionError {
    Syntax(String),
    Runtime(RuntimeError),
    Control,
}
