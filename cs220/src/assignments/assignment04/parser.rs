#![allow(deprecated)]

//! Parser.

use anyhow::{bail, Result};
use etrace::*;
use lazy_static::*;
use pest::iterators::{Pair, Pairs};
use pest::prec_climber::*;
use pest::Parser;

use super::syntax::*;

#[allow(missing_docs)]
#[allow(missing_debug_implementations)]
mod inner {
    use pest_derive::*;

    #[derive(Parser)]
    #[grammar = "assignments/assignment04/syntax.pest"]
    pub(crate) struct SyntaxParser;
}

use inner::*;

lazy_static! {
    static ref CLIMBER: PrecClimber<Rule> = PrecClimber::new(vec![
        // 1. Lowest precedence: Addition and Subtraction
        Operator::new(Rule::add, Assoc::Left) | Operator::new(Rule::subtract, Assoc::Left),
        // 2. Medium precedence: Multiplication and Division
        Operator::new(Rule::multiply, Assoc::Left) | Operator::new(Rule::divide, Assoc::Left),
        // 3. Highest precedence: Power (Exponents)
        Operator::new(Rule::power, Assoc::Right), 
    ]);
}

/// Parses command.
///
/// ## Operator Associativty
///
/// For associativity of each operator, please follow [here](https://docs.rs/pest/latest/pest/prec_climber/struct.PrecClimber.html#examples).
///
/// e.g. `1+2+3` should be parsed into `(1+2)+3`, not `1+(2+3)` because the associativity of
/// plus("add" in our hw) operator is `Left`.
pub fn parse_command(line: &str) -> Result<Command> {
    // parse the input using the rule 'command' specified in syntax.pest
    let mut pairs = SyntaxParser::parse(Rule::command, line)?;

    let mut var_name = None;
    let mut expression_ast = None;

    for pair in pairs {
        match pair.as_rule() {
            Rule::var => {
                var_name = Some(pair.as_str().to_string());
            },
            Rule::expr => {
                // Pass the baton to the helper function!
                expression_ast = Some(parse_expr(pair)?); 
            },
            _ => {} // Ignore SOI, EOI, etc.
        }
    }

    let final_expr = expression_ast.ok_or(anyhow::anyhow!("Missing expression"))?;

    Ok(Command{ 
        variable: var_name,
        expression: final_expr
    })
}

fn parse_expr(pair: Pair<'_, Rule>) -> Result<Expression> {
    let inner_pairs = pair.into_inner();

    CLIMBER.climb(
        inner_pairs,
        |primary_pair| {
            match primary_pair.as_rule() {
            Rule::num => {
                Ok(Expression::Num(primary_pair.as_str().parse()?))
            }
            Rule::var => {
                Ok(Expression::Variable(primary_pair.as_str().to_string()))
            }
            Rule::expr => {
                parse_expr(primary_pair) 
            }
            _ => bail!("Unexpected term rule: {:?}", primary_pair.as_rule())
            }
        },
        |lhs, op_pair, rhs| {
            let left = lhs?;
            let right = rhs?;
            
            match op_pair.as_rule() {
                Rule::add => Ok(Expression::BinOp{
                    op: BinOp::Add, 
                    lhs: Box::new(left),
                    rhs: Box::new(right)
                }),
                Rule::subtract => Ok(Expression::BinOp{
                    op: BinOp::Subtract, 
                    lhs: Box::new(left),
                    rhs: Box::new(right)
                }),
                Rule::multiply => Ok(Expression::BinOp{
                    op: BinOp::Multiply, 
                    lhs: Box::new(left),
                    rhs: Box::new(right)
                }),
                Rule::divide => Ok(Expression::BinOp{
                    op: BinOp::Divide, 
                    lhs: Box::new(left),
                    rhs: Box::new(right)
                }),
                Rule::power => Ok(Expression::BinOp{
                    op: BinOp::Power, 
                    lhs: Box::new(left),
                    rhs: Box::new(right)
                }),
                _ => unreachable!("Pest grammar guarantees only math operators reach here"),
            }
        }
    )
}
