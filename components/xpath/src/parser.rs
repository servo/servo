/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{char, digit1, multispace0};
use nom::combinator::{map, opt, recognize, value};
use nom::error::{Error as NomError, ErrorKind as NomErrorKind, ParseError as NomParseError};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, pair, preceded};
use nom::{AsChar, Finish, IResult, Input, Parser};

use crate::{is_valid_continuation, is_valid_start};

pub(crate) fn parse(input: &str) -> Result<Expr, OwnedParserError> {
    let (_, ast) = expr(input).finish().map_err(OwnedParserError::from)?;
    Ok(ast)
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum Expr {
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Equality(Box<Expr>, EqualityOp, Box<Expr>),
    Relational(Box<Expr>, RelationalOp, Box<Expr>),
    Additive(Box<Expr>, AdditiveOp, Box<Expr>),
    Multiplicative(Box<Expr>, MultiplicativeOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Union(Box<Expr>, Box<Expr>),
    Path(PathExpr),
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum EqualityOp {
    Eq,
    NotEq,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum RelationalOp {
    Lt,
    Gt,
    LtEq,
    GtEq,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum AdditiveOp {
    Add,
    Sub,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum MultiplicativeOp {
    Mul,
    Div,
    Mod,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum UnaryOp {
    Minus,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct PathExpr {
    /// Whether this is an absolute (as opposed to a relative) path expression.
    ///
    /// Absolute paths always start at the starting node, not the context node.
    pub(crate) is_absolute: bool,
    /// Whether this expression starts with `//`. If it does, then an implicit
    /// `descendant-or-self::node()` step will be added.
    pub(crate) is_descendant: bool,
    pub(crate) steps: Vec<StepExpr>,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) struct PredicateListExpr {
    pub(crate) predicates: Vec<Expr>,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) struct FilterExpr {
    pub(crate) primary: PrimaryExpr,
    pub(crate) predicates: PredicateListExpr,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum StepExpr {
    Filter(FilterExpr),
    Axis(AxisStep),
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) struct AxisStep {
    pub(crate) axis: Axis,
    pub(crate) node_test: NodeTest,
    pub(crate) predicates: PredicateListExpr,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum Axis {
    Child,
    Descendant,
    Attribute,
    Self_,
    DescendantOrSelf,
    FollowingSibling,
    Following,
    Namespace,
    Parent,
    Ancestor,
    PrecedingSibling,
    Preceding,
    AncestorOrSelf,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum NodeTest {
    Name(QName),
    Wildcard,
    Kind(KindTest),
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct QName {
    pub(crate) prefix: Option<String>,
    pub(crate) local_part: String,
}

impl std::fmt::Display for QName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.prefix {
            Some(prefix) => write!(f, "{}:{}", prefix, self.local_part),
            None => write!(f, "{}", self.local_part),
        }
    }
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum KindTest {
    PI(Option<String>),
    Comment,
    Text,
    Node,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum PrimaryExpr {
    Literal(Literal),
    Variable(QName),
    Parenthesized(Box<Expr>),
    ContextItem,
    /// We only support the built-in core functions
    Function(CoreFunction),
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum Literal {
    Numeric(NumericLiteral),
    String(String),
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum NumericLiteral {
    Integer(u64),
    Decimal(f64),
}

/// In the DOM we do not support custom functions, so we can enumerate the usable ones
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum CoreFunction {
    // Node Set Functions
    /// last()
    Last,
    /// position()
    Position,
    /// count(node-set)
    Count(Box<Expr>),
    /// id(object)
    Id(Box<Expr>),
    /// local-name(node-set?)
    LocalName(Option<Box<Expr>>),
    /// namespace-uri(node-set?)
    NamespaceUri(Option<Box<Expr>>),
    /// name(node-set?)
    Name(Option<Box<Expr>>),

    // String Functions
    /// string(object?)
    String(Option<Box<Expr>>),
    /// concat(string, string, ...)
    Concat(Vec<Expr>),
    /// starts-with(string, string)
    StartsWith(Box<Expr>, Box<Expr>),
    /// contains(string, string)
    Contains(Box<Expr>, Box<Expr>),
    /// substring-before(string, string)
    SubstringBefore(Box<Expr>, Box<Expr>),
    /// substring-after(string, string)
    SubstringAfter(Box<Expr>, Box<Expr>),
    /// substring(string, number, number?)
    Substring(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    /// string-length(string?)
    StringLength(Option<Box<Expr>>),
    /// normalize-space(string?)
    NormalizeSpace(Option<Box<Expr>>),
    /// translate(string, string, string)
    Translate(Box<Expr>, Box<Expr>, Box<Expr>),

    // Number Functions
    /// number(object?)
    Number(Option<Box<Expr>>),
    /// sum(node-set)
    Sum(Box<Expr>),
    /// floor(number)
    Floor(Box<Expr>),
    /// ceiling(number)
    Ceiling(Box<Expr>),
    /// round(number)
    Round(Box<Expr>),

    // Boolean Functions
    /// boolean(object)
    Boolean(Box<Expr>),
    /// not(boolean)
    Not(Box<Expr>),
    /// true()
    True,
    /// false()
    False,
    /// lang(string)
    Lang(Box<Expr>),
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
fn expr(input: &str) -> IResult<&str, Expr> {
    expr_single(input)
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-Expr>
fn expr_single(input: &str) -> IResult<&str, Expr> {
    or_expr(input)
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-OrExpr>
fn or_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = and_expr(input)?;
    let (input, rest) = many0(preceded(ws(tag("or")), and_expr)).parse(input)?;

    Ok((
        input,
        rest.into_iter()
            .fold(first, |acc, expr| Expr::Or(Box::new(acc), Box::new(expr))),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-AndExpr>
fn and_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = equality_expr(input)?;
    let (input, rest) = many0(preceded(ws(tag("and")), equality_expr)).parse(input)?;

    Ok((
        input,
        rest.into_iter()
            .fold(first, |acc, expr| Expr::And(Box::new(acc), Box::new(expr))),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-EqualityExpr>
fn equality_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = relational_expr(input)?;
    let (input, rest) = many0((
        ws(alt((
            map(tag("="), |_| EqualityOp::Eq),
            map(tag("!="), |_| EqualityOp::NotEq),
        ))),
        relational_expr,
    ))
    .parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            Expr::Equality(Box::new(acc), op, Box::new(expr))
        }),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-RelationalExpr>
fn relational_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = additive_expr(input)?;
    let (input, rest) = many0((
        ws(alt((
            map(tag("<="), |_| RelationalOp::LtEq),
            map(tag(">="), |_| RelationalOp::GtEq),
            map(tag("<"), |_| RelationalOp::Lt),
            map(tag(">"), |_| RelationalOp::Gt),
        ))),
        additive_expr,
    ))
    .parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            Expr::Relational(Box::new(acc), op, Box::new(expr))
        }),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-AdditiveExpr>
fn additive_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = multiplicative_expr(input)?;
    let (input, rest) = many0((
        ws(alt((
            map(tag("+"), |_| AdditiveOp::Add),
            map(tag("-"), |_| AdditiveOp::Sub),
        ))),
        multiplicative_expr,
    ))
    .parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            Expr::Additive(Box::new(acc), op, Box::new(expr))
        }),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-MultiplicativeExpr>
fn multiplicative_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = unary_expr(input)?;
    let (input, rest) = many0((
        ws(alt((
            map(tag("*"), |_| MultiplicativeOp::Mul),
            map(tag("div"), |_| MultiplicativeOp::Div),
            map(tag("mod"), |_| MultiplicativeOp::Mod),
        ))),
        unary_expr,
    ))
    .parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            Expr::Multiplicative(Box::new(acc), op, Box::new(expr))
        }),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-UnaryExpr>
fn unary_expr(input: &str) -> IResult<&str, Expr> {
    let (input, minus_count) = many0(ws(char('-'))).parse(input)?;
    let (input, expr) = union_expr(input)?;

    Ok((
        input,
        (0..minus_count.len()).fold(expr, |acc, _| Expr::Unary(UnaryOp::Minus, Box::new(acc))),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-UnionExpr>
fn union_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = path_expr(input)?;
    let (input, rest) = many0(preceded(ws(char('|')), path_expr)).parse(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, expr| {
            Expr::Union(Box::new(acc), Box::new(expr))
        }),
    ))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-PathExpr>
fn path_expr(input: &str) -> IResult<&str, Expr> {
    ws(alt((
        // "//" RelativePathExpr
        map(
            pair(tag("//"), move |i| relative_path_expr(true, i)),
            |(_, relative_path)| {
                Expr::Path(PathExpr {
                    is_absolute: true,
                    is_descendant: true,
                    steps: relative_path.steps,
                })
            },
        ),
        // "/" RelativePathExpr?
        map(
            pair(char('/'), opt(move |i| relative_path_expr(false, i))),
            |(_, relative_path)| {
                Expr::Path(PathExpr {
                    is_absolute: true,
                    is_descendant: false,
                    steps: relative_path.map(|path| path.steps).unwrap_or_default(),
                })
            },
        ),
        // RelativePathExpr
        map(move |i| relative_path_expr(false, i), Expr::Path),
    )))
    .parse(input)
}

fn relative_path_expr(is_descendant: bool, input: &str) -> IResult<&str, PathExpr> {
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
            all_steps.push(StepExpr::Axis(AxisStep {
                axis: Axis::DescendantOrSelf,
                node_test: NodeTest::Kind(KindTest::Node),
                predicates: PredicateListExpr { predicates: vec![] },
            }));
        }
        all_steps.push(step);
    }

    Ok((
        input,
        PathExpr {
            is_absolute: false,
            is_descendant: false,
            steps: all_steps,
        },
    ))
}

fn step_expr(is_descendant: bool, input: &str) -> IResult<&str, StepExpr> {
    alt((
        map(filter_expr, StepExpr::Filter),
        map(|i| axis_step(is_descendant, i), StepExpr::Axis),
    ))
    .parse(input)
}

fn axis_step(is_descendant: bool, input: &str) -> IResult<&str, AxisStep> {
    let (input, (step, predicates)) = pair(
        alt((move |i| forward_step(is_descendant, i), reverse_step)),
        predicate_list,
    )
    .parse(input)?;

    let (axis, node_test) = step;
    Ok((
        input,
        AxisStep {
            axis,
            node_test,
            predicates,
        },
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
fn filter_expr(input: &str) -> IResult<&str, FilterExpr> {
    let (input, primary) = primary_expr(input)?;
    let (input, predicates) = predicate_list(input)?;

    Ok((
        input,
        FilterExpr {
            primary,
            predicates,
        },
    ))
}

fn predicate_list(input: &str) -> IResult<&str, PredicateListExpr> {
    let (input, predicates) = many0(predicate).parse(input)?;

    Ok((input, PredicateListExpr { predicates }))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-Predicate>
fn predicate(input: &str) -> IResult<&str, Expr> {
    let (input, expr) = delimited(ws(char('[')), expr, ws(char(']'))).parse(input)?;
    Ok((input, expr))
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-PrimaryExpr>
fn primary_expr(input: &str) -> IResult<&str, PrimaryExpr> {
    alt((
        literal,
        var_ref,
        map(parenthesized_expr, |e| {
            PrimaryExpr::Parenthesized(Box::new(e))
        }),
        context_item_expr,
        function_call,
    ))
    .parse(input)
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-Literal>
fn literal(input: &str) -> IResult<&str, PrimaryExpr> {
    map(alt((numeric_literal, string_literal)), |lit| {
        PrimaryExpr::Literal(lit)
    })
    .parse(input)
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-Number>
fn numeric_literal(input: &str) -> IResult<&str, Literal> {
    alt((decimal_literal, integer_literal)).parse(input)
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-VariableReference>
fn var_ref(input: &str) -> IResult<&str, PrimaryExpr> {
    let (input, _) = char('$').parse(input)?;
    let (input, name) = qname(input)?;
    Ok((input, PrimaryExpr::Variable(name)))
}

fn parenthesized_expr(input: &str) -> IResult<&str, Expr> {
    delimited(ws(char('(')), expr, ws(char(')'))).parse(input)
}

fn context_item_expr(input: &str) -> IResult<&str, PrimaryExpr> {
    map(char('.'), |_| PrimaryExpr::ContextItem).parse(input)
}

fn function_call(input: &str) -> IResult<&str, PrimaryExpr> {
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

    Ok((input, PrimaryExpr::Function(core_fn)))
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
        Literal::Numeric(NumericLiteral::Integer(s.parse().unwrap()))
    })
    .parse(input)
}

fn decimal_literal(input: &str) -> IResult<&str, Literal> {
    map(
        recognize((opt(char('-')), opt(digit1), char('.'), digit1)),
        |s: &str| Literal::Numeric(NumericLiteral::Decimal(s.parse().unwrap())),
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
                Expr::Path(PathExpr {
                    is_absolute: false,
                    is_descendant: false,
                    steps: vec![StepExpr::Axis(AxisStep {
                        axis: Axis::Child,
                        node_test: NodeTest::Kind(KindTest::PI(Some("test".to_string()))),
                        predicates: PredicateListExpr {
                            predicates: vec![Expr::Path(PathExpr {
                                is_absolute: false,
                                is_descendant: false,
                                steps: vec![StepExpr::Filter(FilterExpr {
                                    primary: PrimaryExpr::Literal(Literal::Numeric(
                                        NumericLiteral::Integer(2),
                                    )),
                                    predicates: PredicateListExpr { predicates: vec![] },
                                })],
                            })],
                        },
                    })],
                }),
            ),
            (
                "concat('hello', ' ', 'world')",
                Expr::Path(PathExpr {
                    is_absolute: false,
                    is_descendant: false,
                    steps: vec![StepExpr::Filter(FilterExpr {
                        primary: PrimaryExpr::Function(CoreFunction::Concat(vec![
                            Expr::Path(PathExpr {
                                is_absolute: false,
                                is_descendant: false,
                                steps: vec![StepExpr::Filter(FilterExpr {
                                    primary: PrimaryExpr::Literal(Literal::String(
                                        "hello".to_string(),
                                    )),
                                    predicates: PredicateListExpr { predicates: vec![] },
                                })],
                            }),
                            Expr::Path(PathExpr {
                                is_absolute: false,
                                is_descendant: false,
                                steps: vec![StepExpr::Filter(FilterExpr {
                                    primary: PrimaryExpr::Literal(Literal::String(" ".to_string())),
                                    predicates: PredicateListExpr { predicates: vec![] },
                                })],
                            }),
                            Expr::Path(PathExpr {
                                is_absolute: false,
                                is_descendant: false,
                                steps: vec![StepExpr::Filter(FilterExpr {
                                    primary: PrimaryExpr::Literal(Literal::String(
                                        "world".to_string(),
                                    )),
                                    predicates: PredicateListExpr { predicates: vec![] },
                                })],
                            }),
                        ])),
                        predicates: PredicateListExpr { predicates: vec![] },
                    })],
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

    #[test]
    fn test_complex_paths() {
        let cases = vec![
            (
                "//*[contains(@class, 'test')]",
                Expr::Path(PathExpr {
                    is_absolute: true,
                    is_descendant: true,
                    steps: vec![StepExpr::Axis(AxisStep {
                        axis: Axis::DescendantOrSelf,
                        node_test: NodeTest::Wildcard,
                        predicates: PredicateListExpr {
                            predicates: vec![Expr::Path(PathExpr {
                                is_absolute: false,
                                is_descendant: false,
                                steps: vec![StepExpr::Filter(FilterExpr {
                                    primary: PrimaryExpr::Function(CoreFunction::Contains(
                                        Box::new(Expr::Path(PathExpr {
                                            is_absolute: false,
                                            is_descendant: false,
                                            steps: vec![StepExpr::Axis(AxisStep {
                                                axis: Axis::Attribute,
                                                node_test: NodeTest::Name(QName {
                                                    prefix: None,
                                                    local_part: "class".to_string(),
                                                }),
                                                predicates: PredicateListExpr {
                                                    predicates: vec![],
                                                },
                                            })],
                                        })),
                                        Box::new(Expr::Path(PathExpr {
                                            is_absolute: false,
                                            is_descendant: false,
                                            steps: vec![StepExpr::Filter(FilterExpr {
                                                primary: PrimaryExpr::Literal(Literal::String(
                                                    "test".to_string(),
                                                )),
                                                predicates: PredicateListExpr {
                                                    predicates: vec![],
                                                },
                                            })],
                                        })),
                                    )),
                                    predicates: PredicateListExpr { predicates: vec![] },
                                })],
                            })],
                        },
                    })],
                }),
            ),
            (
                "//div[position() > 1]/*[last()]",
                Expr::Path(PathExpr {
                    is_absolute: true,
                    is_descendant: true,
                    steps: vec![
                        StepExpr::Axis(AxisStep {
                            axis: Axis::DescendantOrSelf,
                            node_test: NodeTest::Name(QName {
                                prefix: None,
                                local_part: "div".to_string(),
                            }),
                            predicates: PredicateListExpr {
                                predicates: vec![Expr::Relational(
                                    Box::new(Expr::Path(PathExpr {
                                        is_absolute: false,
                                        is_descendant: false,
                                        steps: vec![StepExpr::Filter(FilterExpr {
                                            primary: PrimaryExpr::Function(CoreFunction::Position),
                                            predicates: PredicateListExpr { predicates: vec![] },
                                        })],
                                    })),
                                    RelationalOp::Gt,
                                    Box::new(Expr::Path(PathExpr {
                                        is_absolute: false,
                                        is_descendant: false,
                                        steps: vec![StepExpr::Filter(FilterExpr {
                                            primary: PrimaryExpr::Literal(Literal::Numeric(
                                                NumericLiteral::Integer(1),
                                            )),
                                            predicates: PredicateListExpr { predicates: vec![] },
                                        })],
                                    })),
                                )],
                            },
                        }),
                        StepExpr::Axis(AxisStep {
                            axis: Axis::Child,
                            node_test: NodeTest::Wildcard,
                            predicates: PredicateListExpr {
                                predicates: vec![Expr::Path(PathExpr {
                                    is_absolute: false,
                                    is_descendant: false,
                                    steps: vec![StepExpr::Filter(FilterExpr {
                                        primary: PrimaryExpr::Function(CoreFunction::Last),
                                        predicates: PredicateListExpr { predicates: vec![] },
                                    })],
                                })],
                            },
                        }),
                    ],
                }),
            ),
            (
                "//mu[@xml:id=\"id1\"]//rho[@title][@xml:lang=\"en-GB\"]",
                Expr::Path(PathExpr {
                    is_absolute: true,
                    is_descendant: true,
                    steps: vec![
                        StepExpr::Axis(AxisStep {
                            axis: Axis::DescendantOrSelf,
                            node_test: NodeTest::Name(QName {
                                prefix: None,
                                local_part: "mu".to_string(),
                            }),
                            predicates: PredicateListExpr {
                                predicates: vec![Expr::Equality(
                                    Box::new(Expr::Path(PathExpr {
                                        is_absolute: false,
                                        is_descendant: false,
                                        steps: vec![StepExpr::Axis(AxisStep {
                                            axis: Axis::Attribute,
                                            node_test: NodeTest::Name(QName {
                                                prefix: Some("xml".to_string()),
                                                local_part: "id".to_string(),
                                            }),
                                            predicates: PredicateListExpr { predicates: vec![] },
                                        })],
                                    })),
                                    EqualityOp::Eq,
                                    Box::new(Expr::Path(PathExpr {
                                        is_absolute: false,
                                        is_descendant: false,
                                        steps: vec![StepExpr::Filter(FilterExpr {
                                            primary: PrimaryExpr::Literal(Literal::String(
                                                "id1".to_string(),
                                            )),
                                            predicates: PredicateListExpr { predicates: vec![] },
                                        })],
                                    })),
                                )],
                            },
                        }),
                        StepExpr::Axis(AxisStep {
                            axis: Axis::DescendantOrSelf, // Represents the second '//'
                            node_test: NodeTest::Kind(KindTest::Node),
                            predicates: PredicateListExpr { predicates: vec![] },
                        }),
                        StepExpr::Axis(AxisStep {
                            axis: Axis::Child,
                            node_test: NodeTest::Name(QName {
                                prefix: None,
                                local_part: "rho".to_string(),
                            }),
                            predicates: PredicateListExpr {
                                predicates: vec![
                                    Expr::Path(PathExpr {
                                        is_absolute: false,
                                        is_descendant: false,
                                        steps: vec![StepExpr::Axis(AxisStep {
                                            axis: Axis::Attribute,
                                            node_test: NodeTest::Name(QName {
                                                prefix: None,
                                                local_part: "title".to_string(),
                                            }),
                                            predicates: PredicateListExpr { predicates: vec![] },
                                        })],
                                    }),
                                    Expr::Equality(
                                        Box::new(Expr::Path(PathExpr {
                                            is_absolute: false,
                                            is_descendant: false,
                                            steps: vec![StepExpr::Axis(AxisStep {
                                                axis: Axis::Attribute,
                                                node_test: NodeTest::Name(QName {
                                                    prefix: Some("xml".to_string()),
                                                    local_part: "lang".to_string(),
                                                }),
                                                predicates: PredicateListExpr {
                                                    predicates: vec![],
                                                },
                                            })],
                                        })),
                                        EqualityOp::Eq,
                                        Box::new(Expr::Path(PathExpr {
                                            is_absolute: false,
                                            is_descendant: false,
                                            steps: vec![StepExpr::Filter(FilterExpr {
                                                primary: PrimaryExpr::Literal(Literal::String(
                                                    "en-GB".to_string(),
                                                )),
                                                predicates: PredicateListExpr {
                                                    predicates: vec![],
                                                },
                                            })],
                                        })),
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
