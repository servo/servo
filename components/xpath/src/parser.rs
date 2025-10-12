/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{char, digit1, multispace0};
use nom::combinator::{map, opt, recognize, value};
use nom::error::{Error as NomError, ErrorKind as NomErrorKind, ParseError as NomParseError};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, pair, preceded};
use nom::{AsChar, Finish, IResult, Input, Parser};

use crate::ast::{
    Axis, BinaryOperator, CoreFunction, Expression, FilterExpression, KindTest, Literal,
    LocationStepExpression, NodeTest, PathExpression, PredicateListExpression, QName,
};
use crate::{is_valid_continuation, is_valid_start};

pub(crate) fn parse(input: &str) -> Result<Expression, OwnedParserError> {
    let (_, ast) = expr(input).finish().map_err(OwnedParserError::from)?;
    Ok(ast)
}

#[derive(Clone, Debug, PartialEq)]
pub struct OwnedParserError {
    pub input: String,
    pub kind: NomErrorKind,
}

impl<'a> From<NomError<&'a str>> for OwnedParserError {
    fn from(err: NomError<&'a str>) -> Self {
        OwnedParserError {
            input: err.input.to_string(),
            kind: err.code,
        }
    }
}

impl std::fmt::Display for OwnedParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "error {:?} at: {}", self.kind, self.input)
    }
}

impl std::error::Error for OwnedParserError {}

/// Top-level parser
fn expr(input: &str) -> IResult<&str, Expression> {
    expr_single(input)
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-Expr>
fn expr_single(input: &str) -> IResult<&str, Expression> {
    or_expr(input)
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-OrExpr>
fn or_expr(input: &str) -> IResult<&str, Expression> {
    let (input, first) = and_expr(input)?;
    let (input, rest) = many0(preceded(ws(tag("or")), and_expr)).parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, expr| {
            Expression::Binary(Box::new(acc), BinaryOperator::Or, Box::new(expr))
        }),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-AndExpr>
fn and_expr(input: &str) -> IResult<&str, Expression> {
    let (input, first) = equality_expr(input)?;
    let (input, rest) = many0(preceded(ws(tag("and")), equality_expr)).parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, expr| {
            Expression::Binary(Box::new(acc), BinaryOperator::And, Box::new(expr))
        }),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-EqualityExpr>
fn equality_expr(input: &str) -> IResult<&str, Expression> {
    let (input, first) = relational_expr(input)?;
    let (input, rest) = many0((
        ws(alt((
            map(tag("="), |_| BinaryOperator::Equal),
            map(tag("!="), |_| BinaryOperator::NotEqual),
        ))),
        relational_expr,
    ))
    .parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            Expression::Binary(Box::new(acc), op, Box::new(expr))
        }),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-RelationalExpr>
fn relational_expr(input: &str) -> IResult<&str, Expression> {
    let (input, first) = additive_expr(input)?;
    let (input, rest) = many0((
        ws(alt((
            map(tag("<="), |_| BinaryOperator::LessThanOrEqual),
            map(tag(">="), |_| BinaryOperator::GreaterThanOrEqual),
            map(tag("<"), |_| BinaryOperator::LessThan),
            map(tag(">"), |_| BinaryOperator::GreaterThan),
        ))),
        additive_expr,
    ))
    .parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            Expression::Binary(Box::new(acc), op, Box::new(expr))
        }),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-AdditiveExpr>
fn additive_expr(input: &str) -> IResult<&str, Expression> {
    let (input, first) = multiplicative_expr(input)?;
    let (input, rest) = many0((
        ws(alt((
            map(tag("+"), |_| BinaryOperator::Add),
            map(tag("-"), |_| BinaryOperator::Subtract),
        ))),
        multiplicative_expr,
    ))
    .parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            Expression::Binary(Box::new(acc), op, Box::new(expr))
        }),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-MultiplicativeExpr>
fn multiplicative_expr(input: &str) -> IResult<&str, Expression> {
    let (input, first) = unary_expr(input)?;
    let (input, rest) = many0((
        ws(alt((
            map(tag("*"), |_| BinaryOperator::Multiply),
            map(tag("div"), |_| BinaryOperator::Divide),
            map(tag("mod"), |_| BinaryOperator::Modulo),
        ))),
        unary_expr,
    ))
    .parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            Expression::Binary(Box::new(acc), op, Box::new(expr))
        }),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-UnaryExpr>
fn unary_expr(input: &str) -> IResult<&str, Expression> {
    let (input, minus_count) = many0(ws(char('-'))).parse(input)?;
    let (input, expr) = union_expr(input)?;

    Ok((
        input,
        (0..minus_count.len()).fold(expr, |acc, _| Expression::Negate(Box::new(acc))),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-UnionExpr>
fn union_expr(input: &str) -> IResult<&str, Expression> {
    let (input, first) = path_expr(input)?;
    let (input, rest) = many0(preceded(ws(char('|')), path_expr)).parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, expr| {
            Expression::Binary(Box::new(acc), BinaryOperator::Union, Box::new(expr))
        }),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-PathExpr>
fn path_expr(input: &str) -> IResult<&str, Expression> {
    ws(alt((
        // "//" RelativePathExpr
        map(
            pair(tag("//"), move |i| relative_path_expr(true, i)),
            |(_, relative_path)| {
                Expression::Path(PathExpression {
                    is_absolute: true,
                    has_implicit_descendant_or_self_step: true,
                    steps: relative_path.steps,
                })
            },
        ),
        // "/" RelativePathExpr?
        map(
            pair(char('/'), opt(move |i| relative_path_expr(false, i))),
            |(_, relative_path)| {
                Expression::Path(PathExpression {
                    is_absolute: true,
                    has_implicit_descendant_or_self_step: false,
                    steps: relative_path.map(|path| path.steps).unwrap_or_default(),
                })
            },
        ),
        // RelativePathExpr
        map(
            move |i| relative_path_expr(false, i),
            |mut relative_path_expression| {
                if relative_path_expression.steps.len() == 1 {
                    relative_path_expression.steps.pop().unwrap()
                } else {
                    Expression::Path(relative_path_expression)
                }
            },
        ),
    )))
    .parse(input)
}

fn relative_path_expr(is_descendant: bool, input: &str) -> IResult<&str, PathExpression> {
    let (input, first) = step_expr(is_descendant, input)?;
    let (input, steps) = many0(pair(
        ws(alt((value(true, tag("//")), value(false, char('/'))))),
        ws(move |i| step_expr(false, i)),
    ))
    .parse(input)?;

    let mut all_steps = vec![first];
    for (implicit_descendant_or_self, step) in steps {
        if implicit_descendant_or_self {
            // Insert an implicit descendant-or-self::node() step
            all_steps.push(Expression::LocationStep(LocationStepExpression {
                axis: Axis::DescendantOrSelf,
                node_test: NodeTest::Kind(KindTest::Node),
                predicate_list: PredicateListExpression { predicates: vec![] },
            }));
        }
        all_steps.push(step);
    }

    Ok((
        input,
        PathExpression {
            is_absolute: false,
            has_implicit_descendant_or_self_step: false,
            steps: all_steps,
        },
    ))
}

fn step_expr(is_descendant: bool, input: &str) -> IResult<&str, Expression> {
    alt((filter_expr, |i| axis_step(is_descendant, i))).parse(input)
}

fn axis_step(is_descendant: bool, input: &str) -> IResult<&str, Expression> {
    let (input, (step, predicates)) = pair(
        alt((move |i| forward_step(is_descendant, i), reverse_step)),
        predicate_list,
    )
    .parse(input)?;

    let (axis, node_test) = step;
    Ok((
        input,
        Expression::LocationStep(LocationStepExpression {
            axis,
            node_test,
            predicate_list: predicates,
        }),
    ))
}

fn forward_step(is_descendant: bool, input: &str) -> IResult<&str, (Axis, NodeTest)> {
    alt((pair(forward_axis, node_test), move |i| {
        abbrev_forward_step(is_descendant, i)
    }))
    .parse(input)
}

fn forward_axis(input: &str) -> IResult<&str, Axis> {
    let (input, axis) = alt((
        value(Axis::Child, tag("child::")),
        value(Axis::Descendant, tag("descendant::")),
        value(Axis::Attribute, tag("attribute::")),
        value(Axis::Self_, tag("self::")),
        value(Axis::DescendantOrSelf, tag("descendant-or-self::")),
        value(Axis::FollowingSibling, tag("following-sibling::")),
        value(Axis::Following, tag("following::")),
        value(Axis::Namespace, tag("namespace::")),
    ))
    .parse(input)?;

    Ok((input, axis))
}

// <https://www.w3.org/TR/1999/REC-xpath-19991116/#path-abbrev>
fn abbrev_forward_step(is_descendant: bool, input: &str) -> IResult<&str, (Axis, NodeTest)> {
    let (input, attr) = opt(char('@')).parse(input)?;
    let (input, test) = node_test(input)?;

    let axis = if attr.is_some() {
        Axis::Attribute
    } else if is_descendant {
        Axis::DescendantOrSelf
    } else {
        Axis::Child
    };
    Ok((input, (axis, test)))
}

fn reverse_step(input: &str) -> IResult<&str, (Axis, NodeTest)> {
    alt((
        // ReverseAxis NodeTest
        pair(reverse_axis, node_test),
        // AbbrevReverseStep
        abbrev_reverse_step,
    ))
    .parse(input)
}

fn reverse_axis(input: &str) -> IResult<&str, Axis> {
    alt((
        value(Axis::Parent, tag("parent::")),
        value(Axis::Ancestor, tag("ancestor::")),
        value(Axis::PrecedingSibling, tag("preceding-sibling::")),
        value(Axis::Preceding, tag("preceding::")),
        value(Axis::AncestorOrSelf, tag("ancestor-or-self::")),
    ))
    .parse(input)
}

fn abbrev_reverse_step(input: &str) -> IResult<&str, (Axis, NodeTest)> {
    map(tag(".."), |_| {
        (Axis::Parent, NodeTest::Kind(KindTest::Node))
    })
    .parse(input)
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-NodeTest>
fn node_test(input: &str) -> IResult<&str, NodeTest> {
    alt((
        map(kind_test, NodeTest::Kind),
        map(name_test, |name| match name {
            NameTest::Wildcard => NodeTest::Wildcard,
            NameTest::QName(qname) => NodeTest::Name(qname),
        }),
    ))
    .parse(input)
}

#[derive(Clone, Debug, PartialEq)]
enum NameTest {
    QName(QName),
    Wildcard,
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-NameTest>
fn name_test(input: &str) -> IResult<&str, NameTest> {
    alt((
        // NCName ":" "*"
        map((ncname, char(':'), char('*')), |(prefix, _, _)| {
            NameTest::QName(QName {
                prefix: Some(prefix.to_string()),
                local_part: "*".to_string(),
            })
        }),
        // "*"
        value(NameTest::Wildcard, char('*')),
        // QName
        map(qname, NameTest::QName),
    ))
    .parse(input)
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-FilterExpr>
fn filter_expr(input: &str) -> IResult<&str, Expression> {
    let (input, primary) = primary_expr(input)?;
    let (input, predicate_list) = predicate_list(input)?;

    if predicate_list.predicates.is_empty() {
        return Ok((input, primary));
    }

    Ok((
        input,
        Expression::Filter(FilterExpression {
            expression: Box::new(primary),
            predicates: predicate_list,
        }),
    ))
}

fn predicate_list(input: &str) -> IResult<&str, PredicateListExpression> {
    let (input, predicates) = many0(predicate).parse(input)?;

    Ok((input, PredicateListExpression { predicates }))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-Predicate>
fn predicate(input: &str) -> IResult<&str, Expression> {
    let (input, expr) = delimited(ws(char('[')), expr, ws(char(']'))).parse(input)?;
    Ok((input, expr))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-PrimaryExpr>
fn primary_expr(input: &str) -> IResult<&str, Expression> {
    alt((
        literal,
        var_ref,
        parenthesized_expr,
        context_item_expr,
        function_call,
    ))
    .parse(input)
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-Literal>
fn literal(input: &str) -> IResult<&str, Expression> {
    map(alt((numeric_literal, string_literal)), |lit| {
        Expression::Literal(lit)
    })
    .parse(input)
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-Number>
fn numeric_literal(input: &str) -> IResult<&str, Literal> {
    alt((decimal_literal, integer_literal)).parse(input)
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-VariableReference>
fn var_ref(input: &str) -> IResult<&str, Expression> {
    let (input, _) = char('$').parse(input)?;
    let (input, name) = qname(input)?;
    Ok((input, Expression::Variable(name)))
}

fn parenthesized_expr(input: &str) -> IResult<&str, Expression> {
    delimited(ws(char('(')), expr, ws(char(')'))).parse(input)
}

fn context_item_expr(input: &str) -> IResult<&str, Expression> {
    map(char('.'), |_| Expression::ContextItem).parse(input)
}

fn function_call(input: &str) -> IResult<&str, Expression> {
    let (input, name) = qname(input)?;
    let (input, args) = delimited(
        ws(char('(')),
        separated_list0(ws(char(',')), expr_single),
        ws(char(')')),
    )
    .parse(input)?;

    // Helper to create error
    let arity_error = || nom::Err::Error(NomError::new(input, NomErrorKind::Verify));

    let core_fn = match name.local_part.as_str() {
        // Node Set Functions
        "last" => CoreFunction::Last,
        "position" => CoreFunction::Position,
        "count" => CoreFunction::Count(Box::new(args.into_iter().next().ok_or_else(arity_error)?)),
        "id" => CoreFunction::Id(Box::new(args.into_iter().next().ok_or_else(arity_error)?)),
        "local-name" => CoreFunction::LocalName(args.into_iter().next().map(Box::new)),
        "namespace-uri" => CoreFunction::NamespaceUri(args.into_iter().next().map(Box::new)),
        "name" => CoreFunction::Name(args.into_iter().next().map(Box::new)),

        // String Functions
        "string" => CoreFunction::String(args.into_iter().next().map(Box::new)),
        "concat" => CoreFunction::Concat(args.into_iter().collect()),
        "starts-with" => {
            let mut args = args.into_iter();
            CoreFunction::StartsWith(
                Box::new(args.next().ok_or_else(arity_error)?),
                Box::new(args.next().ok_or_else(arity_error)?),
            )
        },
        "contains" => {
            let mut args = args.into_iter();
            CoreFunction::Contains(
                Box::new(args.next().ok_or_else(arity_error)?),
                Box::new(args.next().ok_or_else(arity_error)?),
            )
        },
        "substring-before" => {
            let mut args = args.into_iter();
            CoreFunction::SubstringBefore(
                Box::new(args.next().ok_or_else(arity_error)?),
                Box::new(args.next().ok_or_else(arity_error)?),
            )
        },
        "substring-after" => {
            let mut args = args.into_iter();
            CoreFunction::SubstringAfter(
                Box::new(args.next().ok_or_else(arity_error)?),
                Box::new(args.next().ok_or_else(arity_error)?),
            )
        },
        "substring" => {
            let mut args = args.into_iter();
            CoreFunction::Substring(
                Box::new(args.next().ok_or_else(arity_error)?),
                Box::new(args.next().ok_or_else(arity_error)?),
                args.next().map(Box::new),
            )
        },
        "string-length" => CoreFunction::StringLength(args.into_iter().next().map(Box::new)),
        "normalize-space" => CoreFunction::NormalizeSpace(args.into_iter().next().map(Box::new)),
        "translate" => {
            let mut args = args.into_iter();
            CoreFunction::Translate(
                Box::new(args.next().ok_or_else(arity_error)?),
                Box::new(args.next().ok_or_else(arity_error)?),
                Box::new(args.next().ok_or_else(arity_error)?),
            )
        },

        // Number Functions
        "number" => CoreFunction::Number(args.into_iter().next().map(Box::new)),
        "sum" => CoreFunction::Sum(Box::new(args.into_iter().next().ok_or_else(arity_error)?)),
        "floor" => CoreFunction::Floor(Box::new(args.into_iter().next().ok_or_else(arity_error)?)),
        "ceiling" => {
            CoreFunction::Ceiling(Box::new(args.into_iter().next().ok_or_else(arity_error)?))
        },
        "round" => CoreFunction::Round(Box::new(args.into_iter().next().ok_or_else(arity_error)?)),

        // Boolean Functions
        "boolean" => {
            CoreFunction::Boolean(Box::new(args.into_iter().next().ok_or_else(arity_error)?))
        },
        "not" => CoreFunction::Not(Box::new(args.into_iter().next().ok_or_else(arity_error)?)),
        "true" => CoreFunction::True,
        "false" => CoreFunction::False,
        "lang" => CoreFunction::Lang(Box::new(args.into_iter().next().ok_or_else(arity_error)?)),

        // Unknown function
        _ => return Err(nom::Err::Error(NomError::new(input, NomErrorKind::Verify))),
    };

    Ok((input, Expression::Function(core_fn)))
}

fn kind_test(input: &str) -> IResult<&str, KindTest> {
    alt((pi_test, comment_test, text_test, any_kind_test)).parse(input)
}

fn any_kind_test(input: &str) -> IResult<&str, KindTest> {
    map((tag("node"), ws(char('(')), ws(char(')'))), |_| {
        KindTest::Node
    })
    .parse(input)
}

fn text_test(input: &str) -> IResult<&str, KindTest> {
    map((tag("text"), ws(char('(')), ws(char(')'))), |_| {
        KindTest::Text
    })
    .parse(input)
}

fn comment_test(input: &str) -> IResult<&str, KindTest> {
    map((tag("comment"), ws(char('(')), ws(char(')'))), |_| {
        KindTest::Comment
    })
    .parse(input)
}

fn pi_test(input: &str) -> IResult<&str, KindTest> {
    map(
        (
            tag("processing-instruction"),
            ws(char('(')),
            opt(ws(string_literal)),
            ws(char(')')),
        ),
        |(_, _, literal, _)| {
            KindTest::PI(literal.map(|l| match l {
                Literal::String(s) => s,
                _ => unreachable!(),
            }))
        },
    )
    .parse(input)
}

fn ws<'a, F, O, E>(inner: F) -> impl Parser<&'a str, Output = O, Error = E>
where
    E: NomParseError<&'a str>,
    F: Parser<&'a str, Output = O, Error = E>,
{
    delimited(multispace0, inner, multispace0)
}

fn integer_literal(input: &str) -> IResult<&str, Literal> {
    map(recognize((opt(char('-')), digit1)), |s: &str| {
        Literal::Integer(s.parse().unwrap())
    })
    .parse(input)
}

fn decimal_literal(input: &str) -> IResult<&str, Literal> {
    map(
        recognize((opt(char('-')), opt(digit1), char('.'), digit1)),
        |s: &str| Literal::Decimal(s.parse().unwrap()),
    )
    .parse(input)
}

fn string_literal(input: &str) -> IResult<&str, Literal> {
    alt((
        delimited(
            char('"'),
            map(take_while1(|c| c != '"'), |s: &str| {
                Literal::String(s.to_string())
            }),
            char('"'),
        ),
        delimited(
            char('\''),
            map(take_while1(|c| c != '\''), |s: &str| {
                Literal::String(s.to_string())
            }),
            char('\''),
        ),
    ))
    .parse(input)
}

/// <https://www.w3.org/TR/REC-xml-names/#NT-QName>
fn qname(input: &str) -> IResult<&str, QName> {
    let (input, prefix) = opt((ncname, char(':'))).parse(input)?;
    let (input, local) = ncname(input)?;

    Ok((
        input,
        QName {
            prefix: prefix.map(|(p, _)| p.to_string()),
            local_part: local.to_string(),
        },
    ))
}

/// <https://www.w3.org/TR/REC-xml-names/#NT-NCName>
fn ncname(input: &str) -> IResult<&str, &str> {
    fn name_start_character<T, E: NomParseError<T>>(input: T) -> IResult<T, T, E>
    where
        T: Input,
        <T as Input>::Item: AsChar,
    {
        input.split_at_position1_complete(
            |character| !is_valid_start(character.as_char()) || character.as_char() == ':',
            NomErrorKind::OneOf,
        )
    }

    fn name_character<T, E: NomParseError<T>>(input: T) -> IResult<T, T, E>
    where
        T: Input,
        <T as Input>::Item: AsChar,
    {
        input.split_at_position1_complete(
            |character| !is_valid_continuation(character.as_char()) || character.as_char() == ':',
            NomErrorKind::OneOf,
        )
    }

    recognize(pair(name_start_character, many0(name_character))).parse(input)
}

// Test functions to verify the parsers:
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_tests() {
        let cases = vec![
            ("node()", NodeTest::Kind(KindTest::Node)),
            ("text()", NodeTest::Kind(KindTest::Text)),
            ("comment()", NodeTest::Kind(KindTest::Comment)),
            (
                "processing-instruction()",
                NodeTest::Kind(KindTest::PI(None)),
            ),
            (
                "processing-instruction('test')",
                NodeTest::Kind(KindTest::PI(Some("test".to_string()))),
            ),
            ("*", NodeTest::Wildcard),
            (
                "prefix:*",
                NodeTest::Name(QName {
                    prefix: Some("prefix".to_string()),
                    local_part: "*".to_string(),
                }),
            ),
            (
                "div",
                NodeTest::Name(QName {
                    prefix: None,
                    local_part: "div".to_string(),
                }),
            ),
            (
                "ns:div",
                NodeTest::Name(QName {
                    prefix: Some("ns".to_string()),
                    local_part: "div".to_string(),
                }),
            ),
        ];

        for (input, expected) in cases {
            match node_test(input) {
                Ok((remaining, result)) => {
                    assert!(remaining.is_empty(), "Parser didn't consume all input");
                    assert_eq!(result, expected, "{:?} was parsed incorrectly", input);
                },
                Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
            }
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
            match parse(input) {
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
                        axis: Axis::DescendantOrSelf,
                        node_test: NodeTest::Wildcard,
                        predicate_list: PredicateListExpression {
                            predicates: vec![Expression::Function(CoreFunction::Contains(
                                Box::new(Expression::LocationStep(LocationStepExpression {
                                    axis: Axis::Attribute,
                                    node_test: NodeTest::Name(QName {
                                        prefix: None,
                                        local_part: "class".to_owned(),
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
                            axis: Axis::DescendantOrSelf,
                            node_test: NodeTest::Name(QName {
                                prefix: None,
                                local_part: "div".to_owned(),
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
                            axis: Axis::DescendantOrSelf,
                            node_test: NodeTest::Name(QName {
                                prefix: None,
                                local_part: "mu".to_owned(),
                            }),
                            predicate_list: PredicateListExpression {
                                predicates: vec![Expression::Binary(
                                    Box::new(Expression::LocationStep(LocationStepExpression {
                                        axis: Axis::Attribute,
                                        node_test: NodeTest::Name(QName {
                                            prefix: Some("xml".to_owned()),
                                            local_part: "id".to_owned(),
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
                            node_test: NodeTest::Name(QName {
                                prefix: None,
                                local_part: "rho".to_owned(),
                            }),
                            predicate_list: PredicateListExpression {
                                predicates: vec![
                                    Expression::LocationStep(LocationStepExpression {
                                        axis: Axis::Attribute,
                                        node_test: NodeTest::Name(QName {
                                            prefix: None,
                                            local_part: "title".to_owned(),
                                        }),
                                        predicate_list: PredicateListExpression {
                                            predicates: vec![],
                                        },
                                    }),
                                    Expression::Binary(
                                        Box::new(Expression::LocationStep(
                                            LocationStepExpression {
                                                axis: Axis::Attribute,
                                                node_test: NodeTest::Name(QName {
                                                    prefix: Some("xml".to_owned()),
                                                    local_part: "lang".to_owned(),
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
            match parse(input) {
                Ok(result) => {
                    assert_eq!(result, expected, "{:?} was parsed incorrectly", input);
                },
                Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
            }
        }
    }
}
