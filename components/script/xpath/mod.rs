/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use context::EvaluationCtx;
use eval::Evaluatable;
pub(crate) use eval_value::{NodesetHelpers, Value};
use parser::OwnedParserError;
pub(crate) use parser::{Expr, parse as parse_impl};

use super::dom::node::Node;

mod context;
#[allow(dead_code)]
mod eval;
mod eval_function;
#[allow(dead_code)]
mod eval_value;
#[allow(dead_code)]
mod parser;

/// The failure modes of executing an XPath.
#[derive(Debug, PartialEq)]
pub(crate) enum Error {
    /// The XPath was syntactically invalid
    Parsing { source: OwnedParserError },
    /// The XPath could not be executed
    Evaluating { source: eval::Error },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Parsing { source } => write!(f, "Unable to parse XPath: {}", source),
            Error::Evaluating { source } => write!(f, "Unable to evaluate XPath: {}", source),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Parsing { source } => Some(source),
            Error::Evaluating { source } => Some(source),
        }
    }
}

/// Parse an XPath expression from a string
pub(crate) fn parse(xpath: &str) -> Result<Expr, Error> {
    match parse_impl(xpath) {
        Ok(expr) => {
            debug!("Parsed XPath: {:?}", expr);
            Ok(expr)
        },
        Err(e) => {
            debug!("Unable to parse XPath: {}", e);
            Err(Error::Parsing { source: e })
        },
    }
}

/// Evaluate an already-parsed XPath expression
pub(crate) fn evaluate_parsed_xpath(expr: &Expr, context_node: &Node) -> Result<Value, Error> {
    let context = EvaluationCtx::new(context_node);
    match expr.evaluate(&context) {
        Ok(v) => {
            debug!("Evaluated XPath: {:?}", v);
            Ok(v)
        },
        Err(e) => {
            debug!("Unable to evaluate XPath: {}", e);
            Err(Error::Evaluating { source: e })
        },
    }
}
