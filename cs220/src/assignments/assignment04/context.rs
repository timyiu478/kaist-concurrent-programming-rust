//! Calculator.

use std::collections::HashMap;

use anyhow::*;
use etrace::*;

use super::syntax::{BinOp, Command, Expression};

/// Calculator's context.
#[derive(Debug, Default, Clone)]
pub struct Context {
    anonymous_counter: usize,
    variables: HashMap<String, f64>,
}

impl Context {
    /// Creates a new context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current anonymous variable counter.
    pub fn current_counter(&self) -> usize {
        self.anonymous_counter
    }

    /// Calculates the given expression. (We assume the absence of overflow.)
    pub fn calc_expression(&self, expression: &Expression) -> Result<f64> {
        match expression {
            Expression::Num(n) => Ok(*n),
            Expression::Variable(v) => self.variables
            .get(v)
            .copied() // Converts Option<&f64> to Option<f64>
            .ok_or_else(|| anyhow::anyhow!("Undefined variable: {}", v)),
            Expression::BinOp { op, lhs, rhs } => {
                let left = self.calc_expression(lhs)?;
                let right = self.calc_expression(rhs)?;
                match op {
                    BinOp::Add => Ok(left + right),
                    BinOp::Subtract => Ok(left - right),
                    BinOp::Multiply => Ok(left * right),
                    BinOp::Divide => {
                        if right == 0.0 {
                            bail!("Division by zero error!");
                        }
                        Ok(left / right)
                    },
                    BinOp::Power => Ok(left.powf(right)),
                }
            }
        }
    }

    /// Calculates the given command. (We assume the absence of overflow.)
    ///
    /// If there is no variable lhs in the command (i.e. `command.variable = None`), its value
    /// should be stored at `$0`, `$1`, `$2`, ... respectively.
    ///
    /// # Example
    ///
    /// After calculating commad `3 + 5` => Context's variables = `{($0,8)}`
    ///
    /// After calculating commad `v = 3 - 2` => Context's variables = `{($0,8),(v,1))}`
    ///
    /// After calculating commad `3 ^ 2` => Context's variables = `{($0,8),(v,1),($1,9)}`
    pub fn calc_command(&mut self, command: &Command) -> Result<(String, f64)> {
        let val = self.calc_expression(&command.expression)?;
        match &command.variable {
            Some(var_name) => {
                _ = self.variables.insert(var_name.clone(), val);
                Ok((var_name.clone(), val))
            },
            None => {
                let var_name = format!("${}", self.anonymous_counter);
                self.anonymous_counter += 1;
                _ = self.variables.insert(var_name.clone(), val);
                Ok((var_name, val))
            }
        }
    }
}
