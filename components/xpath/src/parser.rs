/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::marker::PhantomData;

use markup5ever::{LocalName, Namespace, Prefix, QualName};

use crate::NamespaceResolver;
use crate::ast::{
    Axis, BinaryOperator, CoreFunction, Expression, FilterExpression, KindTest, Literal,
    LocationStepExpression, NodeTest, PathExpression, PredicateListExpression,
};
use crate::tokenizer::{Error as TokenizerError, LiteralToken, OperatorToken, Token, tokenize};

#[derive(Clone, Debug)]
pub enum Error<E> {
    Tokenization(TokenizerError),
    UnknownFunction,
    ExpectedSeperatorBetweenFunctionArguments,
    TooFewFunctionArguments,
    TooManyFunctionArguments,
    ExpectedClosingParenthesis,
    ExpectedClosingBracket,
    CannotUseVariables,
    UnknownAxis,
    TrailingInput,
    UnknownNodeTest,
    ExpectedNodeTest,
    UnexpectedEndOfInput,
    /// A JS exception that needs to be propagated to the caller.
    JsError(E),
}

impl<E> From<TokenizerError> for Error<E> {
    fn from(value: TokenizerError) -> Self {
        Self::Tokenization(value)
    }
}

pub(crate) fn parse<E, N>(
    input: &str,
    namespace_resolver: Option<N>,
) -> Result<Expression, Error<E>>
where
    E: fmt::Debug,
    N: NamespaceResolver<E>,
{
    let mut parser = Parser::new(input, namespace_resolver)?;
    let root_expression = parser.parse_expression()?;
    if !parser.remaining().is_empty() {
        log::debug!(
            "Found trailing tokens after expression: {:?}",
            parser.remaining()
        );
        return Err(Error::TrailingInput);
    }

    Ok(root_expression)
}

struct Parser<'a, E, N>
where
    N: NamespaceResolver<E>,
{
    tokens: Vec<Token<'a>>,
    position: usize,
    namespace_resolver: Option<N>,
    marker: PhantomData<E>,
}

impl<'a, E, N> Parser<'a, E, N>
where
    E: fmt::Debug,
    N: NamespaceResolver<E>,
{
    fn new(input: &'a str, namespace_resolver: Option<N>) -> Result<Self, TokenizerError> {
        let tokens = tokenize(input)?;
        Ok(Self {
            tokens,
            position: 0,
            namespace_resolver,
            marker: PhantomData,
        })
    }

    fn expect_current_token(&self) -> Result<Token<'a>, Error<E>> {
        self.tokens
            .get(self.position)
            .copied()
            .ok_or(Error::UnexpectedEndOfInput)
    }

    fn peek(&self, n: usize) -> Option<Token<'a>> {
        self.tokens.get(self.position + n).copied()
    }

    fn advance(&mut self, advance_by: usize) {
        self.position += advance_by;
    }

    fn remaining(&self) -> &[Token<'a>] {
        &self.tokens[self.position..]
    }

    fn resolve_qualified_name(&self, prefix: &str) -> Result<Option<Namespace>, E> {
        let Some(namespace_resolver) = self.namespace_resolver.as_ref() else {
            return Ok(None);
        };

        log::debug!("Resolving namespace prefix: {:?}", prefix);
        namespace_resolver
            .resolve_namespace_prefix(Some(prefix))
            .map(|value| value.map(Namespace::from))
    }

    fn advance_if_current_token_equals(&mut self, wanted: Token<'a>) -> bool {
        if self.peek(0).is_some_and(|token| token == wanted) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, Error<E>> {
        let mut result;

        let mut expression_stack: Vec<(Expression, OperatorToken)> = vec![];
        loop {
            let mut negations = 0;
            while self.advance_if_current_token_equals(Token::Operator(OperatorToken::Subtract)) {
                negations += 1;
            }

            result = self.parse_union_expression()?;

            if negations > 1 {
                if negations % 2 == 0 {
                    result = Expression::Function(CoreFunction::Number(Some(Box::new(result))))
                } else {
                    result = Expression::Negate(Box::new(result))
                }
            }

            // If the next token is not an operator then the expression ends here.
            let Some(next_token) = self.peek(0) else {
                break;
            };
            let Token::Operator(current_operator) = next_token else {
                break;
            };
            self.advance(1);

            // Finish all ongoing expressions that have higher precedence
            while let Some((lhs, operator)) = expression_stack
                .pop_if(|(_, operator)| current_operator.precedence() <= operator.precedence())
            {
                result = create_binary_expression(Box::new(lhs), operator, Box::new(result));
            }

            expression_stack.push((result, current_operator));
        }

        // Close any expressions that are still open
        for (lhs, operator) in expression_stack.into_iter().rev() {
            result = create_binary_expression(Box::new(lhs), operator, Box::new(result))
        }

        Ok(result)
    }

    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-UnionExpr>
    fn parse_union_expression(&mut self) -> Result<Expression, Error<E>> {
        let mut result = self.parse_path_expression()?;

        while self.advance_if_current_token_equals(Token::Union) {
            let rhs = self.parse_path_expression()?;
            result = Expression::Binary(Box::new(result), BinaryOperator::Union, Box::new(rhs));
        }

        Ok(result)
    }

    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-PathExpr>
    fn parse_path_expression(&mut self) -> Result<Expression, Error<E>> {
        let current_token = self.expect_current_token()?;

        let is_absolute = matches!(current_token, Token::Parent | Token::Ancestor);
        let has_implicit_descendant_or_self_step = current_token == Token::Ancestor;

        if is_absolute {
            self.advance(1);

            if !self
                .peek(0)
                .is_some_and(|token| token.is_start_of_location_step())
            {
                return Ok(Expression::Path(PathExpression {
                    is_absolute,
                    has_implicit_descendant_or_self_step,
                    steps: vec![],
                }));
            }
        }

        let first_expression = if !is_absolute {
            let expression = self.parse_filter_or_step_expression()?;

            // If there are no further steps in this path expression then return it as-is.
            if !self
                .peek(0)
                .is_some_and(|token| matches!(token, Token::Parent | Token::Ancestor))
            {
                return Ok(expression);
            }

            expression
        } else {
            self.parse_step_expression()?
        };

        let mut path_expression = PathExpression {
            is_absolute,
            has_implicit_descendant_or_self_step,
            steps: vec![first_expression],
        };

        while let Some(current_token) = self.peek(0) {
            match current_token {
                Token::Ancestor => {
                    self.advance(1);

                    // Insert implicit "descendant-or-self" step
                    path_expression
                        .steps
                        .push(Expression::LocationStep(LocationStepExpression {
                            axis: Axis::DescendantOrSelf,
                            node_test: NodeTest::Kind(KindTest::Node),
                            predicate_list: PredicateListExpression { predicates: vec![] },
                        }));
                    true
                },
                Token::Parent => {
                    self.advance(1);
                    false
                },
                _ => {
                    // The path expression ends here.
                    return Ok(Expression::Path(path_expression));
                },
            };

            let step_expression = self.parse_step_expression()?;
            path_expression.steps.push(step_expression);
        }

        Ok(Expression::Path(path_expression))
    }

    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-FilterExpr>
    ///
    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-Step>
    fn parse_filter_or_step_expression(&mut self) -> Result<Expression, Error<E>> {
        let mut expression = match self.expect_current_token()? {
            Token::FunctionCall(name) => {
                self.advance(1);
                self.parse_function_call(name)?
            },
            Token::OpeningParenthesis => {
                self.advance(1);
                let expression = self.parse_expression()?;
                if !self.advance_if_current_token_equals(Token::ClosingParenthesis) {
                    log::debug!("{:?}", self.expect_current_token()?);
                    return Err(Error::ExpectedClosingParenthesis);
                }
                expression
            },
            Token::Literal(literal) => {
                self.advance(1);
                Expression::Literal(literal.into())
            },
            Token::VariableReference(_) => {
                // TODO: Gecko does *something* here. Is it observable?
                // https://searchfox.org/firefox-main/rev/054e2b072785984455b3b59acad9444ba1eeffb4/dom/xslt/xpath/txExprParser.cpp#349
                return Err(Error::CannotUseVariables);
            },
            _ => self.parse_step_expression()?,
        };

        // Parse a potential list of predicates
        let predicate_list = self.parse_predicates()?;
        if !predicate_list.predicates.is_empty() {
            expression = Expression::Filter(FilterExpression {
                expression: Box::new(expression),
                predicates: predicate_list,
            });
        }

        Ok(expression)
    }

    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#section-Location-Steps>
    fn parse_step_expression(&mut self) -> Result<Expression, Error<E>> {
        let axis;
        let mut node_test = None;

        match self.expect_current_token()? {
            Token::AxisIdentifier(axis_name) => {
                self.advance(1);
                axis = match axis_name {
                    "ancestor" => Axis::Ancestor,
                    "ancestor-or-self" => Axis::AncestorOrSelf,
                    "attribute" => Axis::Attribute,
                    "child" => Axis::Child,
                    "descendant" => Axis::Descendant,
                    "descendant-or-self" => Axis::DescendantOrSelf,
                    "following" => Axis::Following,
                    "following-sibling" => Axis::FollowingSibling,
                    "namespace" => Axis::Namespace,
                    "parent" => Axis::Parent,
                    "preceding" => Axis::Preceding,
                    "preceding-sibling" => Axis::PrecedingSibling,
                    "self" => Axis::Self_,
                    _ => {
                        log::debug!("Unknown XPath axis name: {axis_name:?}");
                        return Err(Error::UnknownAxis);
                    },
                };
            },
            Token::AtSign => {
                // This is a shorthand for the attribute axis
                self.advance(1);
                axis = Axis::Attribute;
            },
            Token::ParentNode => {
                self.advance(1);
                axis = Axis::Parent;
                node_test = Some(NodeTest::Kind(KindTest::Node));
            },
            Token::SelfNode => {
                self.advance(1);
                axis = Axis::Self_;
                node_test = Some(NodeTest::Kind(KindTest::Node));
            },
            _ => {
                axis = Axis::Child;
            },
        }

        let node_test = if let Some(node_test) = node_test {
            node_test
        } else if let Token::CName(name_token) = self.expect_current_token()? {
            self.advance(1);

            if name_token.local_name == "*" {
                NodeTest::Wildcard
            } else {
                let namespace = name_token
                    .prefix
                    .map(|prefix| self.resolve_qualified_name(prefix).map_err(Error::JsError))
                    .transpose()?
                    .flatten();

                let qualified_name = QualName {
                    prefix: name_token.prefix.map(Prefix::from),
                    ns: namespace.unwrap_or_default(),
                    local: LocalName::from(name_token.local_name),
                };

                NodeTest::Name(qualified_name)
            }
        } else {
            self.parse_node_test()?
        };

        let predicate_list = self.parse_predicates()?;
        Ok(Expression::LocationStep(LocationStepExpression {
            axis,
            node_test,
            predicate_list,
        }))
    }

    fn parse_node_test(&mut self) -> Result<NodeTest, Error<E>> {
        let kind_test = match self.expect_current_token()? {
            Token::CommentTest => {
                self.advance(1);
                KindTest::Comment
            },
            Token::NodeTest => {
                self.advance(1);
                KindTest::Node
            },
            Token::ProcessingInstructionTest => {
                self.advance(1);
                let name = if let Token::Literal(LiteralToken::String(name)) =
                    self.expect_current_token()?
                {
                    self.advance(1);
                    Some(name)
                } else {
                    None
                };
                KindTest::PI(name.map(String::from))
            },
            Token::TextTest => {
                self.advance(1);
                KindTest::Text
            },
            _ => {
                return Err(Error::ExpectedNodeTest);
            },
        };

        if !self.advance_if_current_token_equals(Token::ClosingParenthesis) {
            return Err(Error::TooManyFunctionArguments);
        }

        Ok(NodeTest::Kind(kind_test))
    }

    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#predicates>
    fn parse_predicates(&mut self) -> Result<PredicateListExpression, Error<E>> {
        let mut predicates = vec![];
        while self.advance_if_current_token_equals(Token::OpeningBracket) {
            let expression = self.parse_expression()?;
            predicates.push(expression);
            if !self.advance_if_current_token_equals(Token::ClosingBracket) {
                return Err(Error::ExpectedClosingBracket);
            }
        }
        Ok(PredicateListExpression { predicates })
    }

    fn parse_function_call(&mut self, function_name: &str) -> Result<Expression, Error<E>> {
        struct ArgumentIterator<'a, 'b, E, N>
        where
            N: NamespaceResolver<E>,
        {
            parser: &'b mut Parser<'a, E, N>,
            done: bool,
        }

        impl<'a, 'b, E, N> ArgumentIterator<'a, 'b, E, N>
        where
            E: fmt::Debug,
            N: NamespaceResolver<E>,
        {
            fn maybe_next(&mut self) -> Result<Option<Expression>, Error<E>> {
                if self.done {
                    return Ok(None);
                }
                let expression = self.parser.parse_expression()?;
                if self
                    .parser
                    .advance_if_current_token_equals(Token::ClosingParenthesis)
                {
                    self.done = true;
                } else if !self.parser.advance_if_current_token_equals(Token::Comma) {
                    log::debug!("{:?}", self.parser.peek(0));
                    return Err(Error::ExpectedSeperatorBetweenFunctionArguments);
                }

                Ok(Some(expression))
            }

            fn next(&mut self) -> Result<Expression, Error<E>> {
                self.maybe_next()
                    .and_then(|maybe_argument| maybe_argument.ok_or(Error::TooFewFunctionArguments))
            }
        }

        let mut arguments = ArgumentIterator {
            done: self.advance_if_current_token_equals(Token::ClosingParenthesis),
            parser: self,
        };

        let core_fn = match function_name {
            // Node Set Functions
            "last" => CoreFunction::Last,
            "position" => CoreFunction::Position,
            "count" => CoreFunction::Count(Box::new(arguments.next()?)),
            "id" => CoreFunction::Id(Box::new(arguments.next()?)),
            "local-name" => CoreFunction::LocalName(arguments.maybe_next()?.map(Box::new)),
            "namespace-uri" => CoreFunction::NamespaceUri(arguments.maybe_next()?.map(Box::new)),
            "name" => CoreFunction::Name(arguments.maybe_next()?.map(Box::new)),

            // String Functions
            "string" => CoreFunction::String(arguments.maybe_next()?.map(Box::new)),
            "concat" => {
                let mut args = vec![];
                while let Some(argument) = arguments.maybe_next()? {
                    args.push(argument);
                }
                CoreFunction::Concat(args)
            },
            "starts-with" => {
                CoreFunction::StartsWith(Box::new(arguments.next()?), Box::new(arguments.next()?))
            },
            "contains" => {
                CoreFunction::Contains(Box::new(arguments.next()?), Box::new(arguments.next()?))
            },
            "substring-before" => CoreFunction::SubstringBefore(
                Box::new(arguments.next()?),
                Box::new(arguments.next()?),
            ),
            "substring-after" => CoreFunction::SubstringAfter(
                Box::new(arguments.next()?),
                Box::new(arguments.next()?),
            ),
            "substring" => CoreFunction::Substring(
                Box::new(arguments.next()?),
                Box::new(arguments.next()?),
                arguments.maybe_next()?.map(Box::new),
            ),
            "string-length" => CoreFunction::StringLength(arguments.maybe_next()?.map(Box::new)),
            "normalize-space" => {
                CoreFunction::NormalizeSpace(arguments.maybe_next()?.map(Box::new))
            },
            "translate" => CoreFunction::Translate(
                Box::new(arguments.next()?),
                Box::new(arguments.next()?),
                Box::new(arguments.next()?),
            ),

            // Number Functions
            "number" => CoreFunction::Number(arguments.maybe_next()?.map(Box::new)),
            "sum" => CoreFunction::Sum(Box::new(arguments.next()?)),
            "floor" => CoreFunction::Floor(Box::new(arguments.next()?)),
            "ceiling" => CoreFunction::Ceiling(Box::new(arguments.next()?)),
            "round" => CoreFunction::Round(Box::new(arguments.next()?)),

            // Boolean Functions
            "boolean" => CoreFunction::Boolean(Box::new(arguments.next()?)),
            "not" => CoreFunction::Not(Box::new(arguments.next()?)),
            "true" => CoreFunction::True,
            "false" => CoreFunction::False,
            "lang" => CoreFunction::Lang(Box::new(arguments.next()?)),

            // Unknown function
            _ => return Err(Error::UnknownFunction),
        };

        // Ensure that there are no more arguments left
        if !arguments.done {
            return Err(Error::TooManyFunctionArguments);
        }

        Ok(Expression::Function(core_fn))
    }
}

fn create_binary_expression(
    lhs: Box<Expression>,
    operator: OperatorToken,
    rhs: Box<Expression>,
) -> Expression {
    let binary_operator = match operator {
        OperatorToken::And => BinaryOperator::And,
        OperatorToken::Or => BinaryOperator::Or,
        OperatorToken::Multiply => BinaryOperator::Multiply,
        OperatorToken::Divide => BinaryOperator::Divide,
        OperatorToken::Modulo => BinaryOperator::Modulo,
        OperatorToken::Add => BinaryOperator::Add,
        OperatorToken::Subtract => BinaryOperator::Subtract,
        OperatorToken::Equal => BinaryOperator::Equal,
        OperatorToken::NotEqual => BinaryOperator::NotEqual,
        OperatorToken::GreaterThan => BinaryOperator::GreaterThan,
        OperatorToken::GreaterThanOrEqual => BinaryOperator::GreaterThanOrEqual,
        OperatorToken::LessThan => BinaryOperator::LessThan,
        OperatorToken::LessThanOrEqual => BinaryOperator::LessThanOrEqual,
    };

    Expression::Binary(lhs, binary_operator, rhs)
}

impl<'a> From<LiteralToken<'a>> for Literal {
    fn from(value: LiteralToken<'a>) -> Self {
        match value {
            LiteralToken::Integer(integer) => Self::Integer(integer),
            LiteralToken::Decimal(float) => Self::Decimal(float),
            LiteralToken::String(string) => Self::String(string.to_owned()),
        }
    }
}

// Test functions to verify the parsers:
#[cfg(test)]
mod tests {
    use markup5ever::{LocalName, QualName, local_name, namespace_prefix, ns};

    use super::*;
    use crate::NamespaceResolver;

    #[derive(Clone)]
    struct DummyNamespaceResolver;

    impl NamespaceResolver<()> for DummyNamespaceResolver {
        fn resolve_namespace_prefix(&self, _: Option<&str>) -> Result<Option<String>, ()> {
            Ok(Some("http://www.w3.org/1999/xhtml".to_owned()))
        }
    }

    #[test]
    fn test_filter_expr() {
        let cases = vec![
            (
                "processing-instruction('test')[2]",
                Expression::LocationStep(LocationStepExpression {
                    axis: Axis::Child,
                    node_test: NodeTest::Kind(KindTest::PI(Some("test".to_string()))),
                    predicate_list: PredicateListExpression {
                        predicates: vec![Expression::Literal(Literal::Integer(2))],
                    },
                }),
            ),
            (
                "concat('hello', ' ', 'world')",
                Expression::Function(CoreFunction::Concat(vec![
                    Expression::Literal(Literal::String("hello".to_string())),
                    Expression::Literal(Literal::String(" ".to_string())),
                    Expression::Literal(Literal::String("world".to_string())),
                ])),
            ),
        ];

        for (input, expected) in cases {
            match parse(input, Some(DummyNamespaceResolver)) {
                Ok(result) => {
                    assert_eq!(result, expected, "{:?} was parsed incorrectly", input);
                },
                Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn test_complex_paths() {
        let cases = vec![
            (
                "//*[contains(@class, 'test')]",
                Expression::Path(PathExpression {
                    is_absolute: true,
                    has_implicit_descendant_or_self_step: true,
                    steps: vec![Expression::LocationStep(LocationStepExpression {
                        axis: Axis::Child,
                        node_test: NodeTest::Wildcard,
                        predicate_list: PredicateListExpression {
                            predicates: vec![Expression::Function(CoreFunction::Contains(
                                Box::new(Expression::LocationStep(LocationStepExpression {
                                    axis: Axis::Attribute,
                                    node_test: NodeTest::Name(QualName {
                                        prefix: None,
                                        ns: ns!(),
                                        local: local_name!("class"),
                                    }),
                                    predicate_list: PredicateListExpression { predicates: vec![] },
                                })),
                                Box::new(Expression::Literal(Literal::String("test".to_owned()))),
                            ))],
                        },
                    })],
                }),
            ),
            (
                "//div[position() > 1]/*[last()]",
                Expression::Path(PathExpression {
                    is_absolute: true,
                    has_implicit_descendant_or_self_step: true,
                    steps: vec![
                        Expression::LocationStep(LocationStepExpression {
                            axis: Axis::Child,
                            node_test: NodeTest::Name(QualName {
                                prefix: None,
                                ns: ns!(),
                                local: local_name!("div"),
                            }),
                            predicate_list: PredicateListExpression {
                                predicates: vec![Expression::Binary(
                                    Box::new(Expression::Function(CoreFunction::Position)),
                                    BinaryOperator::GreaterThan,
                                    Box::new(Expression::Literal(Literal::Integer(1))),
                                )],
                            },
                        }),
                        Expression::LocationStep(LocationStepExpression {
                            axis: Axis::Child,
                            node_test: NodeTest::Wildcard,
                            predicate_list: PredicateListExpression {
                                predicates: vec![Expression::Function(CoreFunction::Last)],
                            },
                        }),
                    ],
                }),
            ),
            (
                "//mu[@xml:id=\"id1\"]//rho[@title][@xml:lang=\"en-GB\"]",
                Expression::Path(PathExpression {
                    is_absolute: true,
                    has_implicit_descendant_or_self_step: true,
                    steps: vec![
                        Expression::LocationStep(LocationStepExpression {
                            axis: Axis::Child,
                            node_test: NodeTest::Name(QualName {
                                prefix: None,
                                ns: ns!(),
                                local: LocalName::from("mu"),
                            }),
                            predicate_list: PredicateListExpression {
                                predicates: vec![Expression::Binary(
                                    Box::new(Expression::LocationStep(LocationStepExpression {
                                        axis: Axis::Attribute,
                                        node_test: NodeTest::Name(QualName {
                                            prefix: Some(namespace_prefix!("xml")),
                                            ns: ns!(html),
                                            local: local_name!("id"),
                                        }),
                                        predicate_list: PredicateListExpression {
                                            predicates: vec![],
                                        },
                                    })),
                                    BinaryOperator::Equal,
                                    Box::new(Expression::Literal(Literal::String(
                                        "id1".to_owned(),
                                    ))),
                                )],
                            },
                        }),
                        Expression::LocationStep(LocationStepExpression {
                            axis: Axis::DescendantOrSelf,
                            node_test: NodeTest::Kind(KindTest::Node),
                            predicate_list: PredicateListExpression { predicates: vec![] },
                        }),
                        Expression::LocationStep(LocationStepExpression {
                            axis: Axis::Child,
                            node_test: NodeTest::Name(QualName {
                                prefix: None,
                                ns: ns!(),
                                local: LocalName::from("rho"),
                            }),
                            predicate_list: PredicateListExpression {
                                predicates: vec![
                                    Expression::LocationStep(LocationStepExpression {
                                        axis: Axis::Attribute,
                                        node_test: NodeTest::Name(QualName {
                                            prefix: None,
                                            ns: ns!(),
                                            local: local_name!("title"),
                                        }),
                                        predicate_list: PredicateListExpression {
                                            predicates: vec![],
                                        },
                                    }),
                                    Expression::Binary(
                                        Box::new(Expression::LocationStep(
                                            LocationStepExpression {
                                                axis: Axis::Attribute,
                                                node_test: NodeTest::Name(QualName {
                                                    prefix: Some(namespace_prefix!("xml")),
                                                    ns: ns!(html),
                                                    local: local_name!("lang"),
                                                }),
                                                predicate_list: PredicateListExpression {
                                                    predicates: vec![],
                                                },
                                            },
                                        )),
                                        BinaryOperator::Equal,
                                        Box::new(Expression::Literal(Literal::String(
                                            "en-GB".to_owned(),
                                        ))),
                                    ),
                                ],
                            },
                        }),
                    ],
                }),
            ),
        ];

        for (input, expected) in cases {
            match parse(input, Some(DummyNamespaceResolver)) {
                Ok(result) => {
                    assert_eq!(result, expected, "{:?} was parsed incorrectly", input);
                },
                Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
            }
        }
    }

    #[test]
    fn parse_expression_in_parenthesis() {
        let test_case = "(./span)";
        let expected = Expression::Path(PathExpression {
            is_absolute: false,
            has_implicit_descendant_or_self_step: false,
            steps: vec![
                Expression::LocationStep(LocationStepExpression {
                    axis: Axis::Self_,
                    node_test: NodeTest::Kind(KindTest::Node),
                    predicate_list: PredicateListExpression { predicates: vec![] },
                }),
                Expression::LocationStep(LocationStepExpression {
                    axis: Axis::Child,
                    node_test: NodeTest::Name(QualName {
                        prefix: None,
                        ns: ns!(),
                        local: local_name!("span"),
                    }),
                    predicate_list: PredicateListExpression { predicates: vec![] },
                }),
            ],
        });
        match parse(test_case, Some(DummyNamespaceResolver)) {
            Ok(result) => {
                assert_eq!(result, expected, "{:?} was parsed incorrectly", test_case);
            },
            Err(e) => panic!("Failed to parse '{}': {:?}", test_case, e),
        }
    }
}
