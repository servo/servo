/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use context::EvaluationCtx;
use eval::{Error as EvaluationError, Evaluatable};
pub(crate) use eval_value::{NodesetHelpers, Value};
pub(crate) use parser::{Expr, parse as parse_impl};

mod context;
mod eval;
mod eval_function;
mod eval_value;
mod parser;

/// Parse an XPath expression from a string
pub(crate) fn parse(xpath: &str) -> Fallible<Expr> {
    match parse_impl(xpath) {
        Ok(expr) => {
            debug!("Parsed XPath: {expr:?}");
            Ok(expr)
        },
        Err(error) => {
            debug!("Unable to parse XPath: {error}");
            Err(Error::Operation)
        },
    }
}

/// Evaluate an already-parsed XPath expression
pub(crate) fn evaluate_parsed_xpath(
    expr: &Expr,
    context_node: &Node,
    resolver: Option<Rc<XPathNSResolver>>,
) -> Fallible<Value> {
    let context = EvaluationCtx::new(context_node, resolver);
    match expr.evaluate(&context) {
        Ok(value) => {
            debug!("Evaluated XPath: {value:?}");
            Ok(value)
        },
        Err(error) => {
            debug!("Unable to evaluate XPath: {error}");

            let error = match error {
                EvaluationError::JsException(exception) => exception,
                _ => JsError::Operation,
            };

            Err(error)
        },
    }
}
