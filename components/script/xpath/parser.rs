/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{alpha1, alphanumeric1, char, digit1, multispace0};
use nom::combinator::{map, opt, recognize, value};
use nom::error::{Error as NomError, ErrorKind as NomErrorKind};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::{Finish, IResult};

pub(crate) fn parse(input: &str) -> Result<Expr, OwnedParserError> {
    let (_, ast) = expr(input).finish().map_err(OwnedParserError::from)?;
    Ok(ast)
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum Expr {
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
pub(crate) enum EqualityOp {
    Eq,
    NotEq,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum RelationalOp {
    Lt,
    Gt,
    LtEq,
    GtEq,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum AdditiveOp {
    Add,
    Sub,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum MultiplicativeOp {
    Mul,
    Div,
    Mod,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) enum UnaryOp {
    Minus,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) struct PathExpr {
    pub(crate) is_absolute: bool,
    pub(crate) is_descendant: bool,
    pub(crate) steps: Vec<StepExpr>,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) struct PredicateListExpr {
    pub(crate) predicates: Vec<PredicateExpr>,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub(crate) struct PredicateExpr {
    pub(crate) expr: Expr,
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
pub(crate) struct QName {
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

impl CoreFunction {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            CoreFunction::Last => "last",
            CoreFunction::Position => "position",
            CoreFunction::Count(_) => "count",
            CoreFunction::Id(_) => "id",
            CoreFunction::LocalName(_) => "local-name",
            CoreFunction::NamespaceUri(_) => "namespace-uri",
            CoreFunction::Name(_) => "name",
            CoreFunction::String(_) => "string",
            CoreFunction::Concat(_) => "concat",
            CoreFunction::StartsWith(_, _) => "starts-with",
            CoreFunction::Contains(_, _) => "contains",
            CoreFunction::SubstringBefore(_, _) => "substring-before",
            CoreFunction::SubstringAfter(_, _) => "substring-after",
            CoreFunction::Substring(_, _, _) => "substring",
            CoreFunction::StringLength(_) => "string-length",
            CoreFunction::NormalizeSpace(_) => "normalize-space",
            CoreFunction::Translate(_, _, _) => "translate",
            CoreFunction::Number(_) => "number",
            CoreFunction::Sum(_) => "sum",
            CoreFunction::Floor(_) => "floor",
            CoreFunction::Ceiling(_) => "ceiling",
            CoreFunction::Round(_) => "round",
            CoreFunction::Boolean(_) => "boolean",
            CoreFunction::Not(_) => "not",
            CoreFunction::True => "true",
            CoreFunction::False => "false",
            CoreFunction::Lang(_) => "lang",
        }
    }

    pub(crate) fn min_args(&self) -> usize {
        match self {
            // No args
            CoreFunction::Last |
            CoreFunction::Position |
            CoreFunction::True |
            CoreFunction::False => 0,

            // Optional single arg
            CoreFunction::LocalName(_) |
            CoreFunction::NamespaceUri(_) |
            CoreFunction::Name(_) |
            CoreFunction::String(_) |
            CoreFunction::StringLength(_) |
            CoreFunction::NormalizeSpace(_) |
            CoreFunction::Number(_) => 0,

            // Required single arg
            CoreFunction::Count(_) |
            CoreFunction::Id(_) |
            CoreFunction::Sum(_) |
            CoreFunction::Floor(_) |
            CoreFunction::Ceiling(_) |
            CoreFunction::Round(_) |
            CoreFunction::Boolean(_) |
            CoreFunction::Not(_) |
            CoreFunction::Lang(_) => 1,

            // Required two args
            CoreFunction::StartsWith(_, _) |
            CoreFunction::Contains(_, _) |
            CoreFunction::SubstringBefore(_, _) |
            CoreFunction::SubstringAfter(_, _) => 2,

            // Special cases
            CoreFunction::Concat(_) => 2,          // Minimum 2 args
            CoreFunction::Substring(_, _, _) => 2, // 2 or 3 args
            CoreFunction::Translate(_, _, _) => 3, // Exactly 3 args
        }
    }

    pub(crate) fn max_args(&self) -> Option<usize> {
        match self {
            // No args
            CoreFunction::Last |
            CoreFunction::Position |
            CoreFunction::True |
            CoreFunction::False => Some(0),

            // Optional single arg (0 or 1)
            CoreFunction::LocalName(_) |
            CoreFunction::NamespaceUri(_) |
            CoreFunction::Name(_) |
            CoreFunction::String(_) |
            CoreFunction::StringLength(_) |
            CoreFunction::NormalizeSpace(_) |
            CoreFunction::Number(_) => Some(1),

            // Exactly one arg
            CoreFunction::Count(_) |
            CoreFunction::Id(_) |
            CoreFunction::Sum(_) |
            CoreFunction::Floor(_) |
            CoreFunction::Ceiling(_) |
            CoreFunction::Round(_) |
            CoreFunction::Boolean(_) |
            CoreFunction::Not(_) |
            CoreFunction::Lang(_) => Some(1),

            // Exactly two args
            CoreFunction::StartsWith(_, _) |
            CoreFunction::Contains(_, _) |
            CoreFunction::SubstringBefore(_, _) |
            CoreFunction::SubstringAfter(_, _) => Some(2),

            // Special cases
            CoreFunction::Concat(_) => None, // Unlimited args
            CoreFunction::Substring(_, _, _) => Some(3), // 2 or 3 args
            CoreFunction::Translate(_, _, _) => Some(3), // Exactly 3 args
        }
    }

    /// Returns true if the number of arguments is valid for this function
    pub(crate) fn is_valid_arity(&self, num_args: usize) -> bool {
        let min = self.min_args();
        let max = self.max_args();

        num_args >= min && max.is_none_or(|max| num_args <= max)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OwnedParserError {
    input: String,
    kind: NomErrorKind,
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

fn expr_single(input: &str) -> IResult<&str, Expr> {
    or_expr(input)
}

fn or_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = and_expr(input)?;
    let (input, rest) = many0(preceded(ws(tag("or")), and_expr))(input)?;

    Ok((
        input,
        rest.into_iter()
            .fold(first, |acc, expr| Expr::Or(Box::new(acc), Box::new(expr))),
    ))
}

fn and_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = equality_expr(input)?;
    let (input, rest) = many0(preceded(ws(tag("and")), equality_expr))(input)?;

    Ok((
        input,
        rest.into_iter()
            .fold(first, |acc, expr| Expr::And(Box::new(acc), Box::new(expr))),
    ))
}

fn equality_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = relational_expr(input)?;
    let (input, rest) = many0(tuple((
        ws(alt((
            map(tag("="), |_| EqualityOp::Eq),
            map(tag("!="), |_| EqualityOp::NotEq),
        ))),
        relational_expr,
    )))(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            Expr::Equality(Box::new(acc), op, Box::new(expr))
        }),
    ))
}

fn relational_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = additive_expr(input)?;
    let (input, rest) = many0(tuple((
        ws(alt((
            map(tag("<="), |_| RelationalOp::LtEq),
            map(tag(">="), |_| RelationalOp::GtEq),
            map(tag("<"), |_| RelationalOp::Lt),
            map(tag(">"), |_| RelationalOp::Gt),
        ))),
        additive_expr,
    )))(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            Expr::Relational(Box::new(acc), op, Box::new(expr))
        }),
    ))
}

fn additive_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = multiplicative_expr(input)?;
    let (input, rest) = many0(tuple((
        ws(alt((
            map(tag("+"), |_| AdditiveOp::Add),
            map(tag("-"), |_| AdditiveOp::Sub),
        ))),
        multiplicative_expr,
    )))(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            Expr::Additive(Box::new(acc), op, Box::new(expr))
        }),
    ))
}

fn multiplicative_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = unary_expr(input)?;
    let (input, rest) = many0(tuple((
        ws(alt((
            map(tag("*"), |_| MultiplicativeOp::Mul),
            map(tag("div"), |_| MultiplicativeOp::Div),
            map(tag("mod"), |_| MultiplicativeOp::Mod),
        ))),
        unary_expr,
    )))(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, (op, expr)| {
            Expr::Multiplicative(Box::new(acc), op, Box::new(expr))
        }),
    ))
}

fn unary_expr(input: &str) -> IResult<&str, Expr> {
    let (input, minus_count) = many0(ws(char('-')))(input)?;
    let (input, expr) = union_expr(input)?;

    Ok((
        input,
        (0..minus_count.len()).fold(expr, |acc, _| Expr::Unary(UnaryOp::Minus, Box::new(acc))),
    ))
}

fn union_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = path_expr(input)?;
    let (input, rest) = many0(preceded(ws(char('|')), path_expr))(input)?;

    Ok((
        input,
        rest.into_iter().fold(first, |acc, expr| {
            Expr::Union(Box::new(acc), Box::new(expr))
        }),
    ))
}

fn path_expr(input: &str) -> IResult<&str, Expr> {
    alt((
        // "//" RelativePathExpr
        map(pair(tag("//"), relative_path_expr), |(_, rel_path)| {
            Expr::Path(PathExpr {
                is_absolute: true,
                is_descendant: true,
                steps: match rel_path {
                    Expr::Path(p) => p.steps,
                    _ => unreachable!(),
                },
            })
        }),
        // "/" RelativePathExpr?
        map(pair(char('/'), opt(relative_path_expr)), |(_, rel_path)| {
            Expr::Path(PathExpr {
                is_absolute: true,
                is_descendant: false,
                steps: rel_path
                    .map(|p| match p {
                        Expr::Path(p) => p.steps,
                        _ => unreachable!(),
                    })
                    .unwrap_or_default(),
            })
        }),
        // RelativePathExpr
        relative_path_expr,
    ))(input)
}

fn relative_path_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = step_expr(input)?;
    let (input, steps) = many0(pair(
        // ("/" | "//")
        ws(alt((value(false, char('/')), value(true, tag("//"))))),
        step_expr,
    ))(input)?;

    let mut all_steps = vec![first];
    for (is_descendant, step) in steps {
        if is_descendant {
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
        Expr::Path(PathExpr {
            is_absolute: false,
            is_descendant: false,
            steps: all_steps,
        }),
    ))
}

fn step_expr(input: &str) -> IResult<&str, StepExpr> {
    alt((
        map(filter_expr, StepExpr::Filter),
        map(axis_step, StepExpr::Axis),
    ))(input)
}

fn axis_step(input: &str) -> IResult<&str, AxisStep> {
    let (input, (step, predicates)) =
        pair(alt((forward_step, reverse_step)), predicate_list)(input)?;

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

fn forward_step(input: &str) -> IResult<&str, (Axis, NodeTest)> {
    alt((
        // ForwardAxis NodeTest
        pair(forward_axis, node_test),
        // AbbrevForwardStep
        abbrev_forward_step,
    ))(input)
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
    ))(input)?;

    Ok((input, axis))
}

fn abbrev_forward_step(input: &str) -> IResult<&str, (Axis, NodeTest)> {
    let (input, attr) = opt(char('@'))(input)?;
    let (input, test) = node_test(input)?;

    Ok((
        input,
        (
            if attr.is_some() {
                Axis::Attribute
            } else {
                Axis::Child
            },
            test,
        ),
    ))
}

fn reverse_step(input: &str) -> IResult<&str, (Axis, NodeTest)> {
    alt((
        // ReverseAxis NodeTest
        pair(reverse_axis, node_test),
        // AbbrevReverseStep
        abbrev_reverse_step,
    ))(input)
}

fn reverse_axis(input: &str) -> IResult<&str, Axis> {
    alt((
        value(Axis::Parent, tag("parent::")),
        value(Axis::Ancestor, tag("ancestor::")),
        value(Axis::PrecedingSibling, tag("preceding-sibling::")),
        value(Axis::Preceding, tag("preceding::")),
        value(Axis::AncestorOrSelf, tag("ancestor-or-self::")),
    ))(input)
}

fn abbrev_reverse_step(input: &str) -> IResult<&str, (Axis, NodeTest)> {
    map(tag(".."), |_| {
        (Axis::Parent, NodeTest::Kind(KindTest::Node))
    })(input)
}

fn node_test(input: &str) -> IResult<&str, NodeTest> {
    alt((
        map(kind_test, NodeTest::Kind),
        map(name_test, |name| match name {
            NameTest::Wildcard => NodeTest::Wildcard,
            NameTest::QName(qname) => NodeTest::Name(qname),
        }),
    ))(input)
}

#[derive(Clone, Debug, PartialEq)]
enum NameTest {
    QName(QName),
    Wildcard,
}

fn name_test(input: &str) -> IResult<&str, NameTest> {
    alt((
        // NCName ":" "*"
        map(tuple((ncname, char(':'), char('*'))), |(prefix, _, _)| {
            NameTest::QName(QName {
                prefix: Some(prefix.to_string()),
                local_part: "*".to_string(),
            })
        }),
        // "*"
        value(NameTest::Wildcard, char('*')),
        // QName
        map(qname, NameTest::QName),
    ))(input)
}

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
    let (input, predicates) = many0(predicate)(input)?;
    Ok((input, PredicateListExpr { predicates }))
}

fn predicate(input: &str) -> IResult<&str, PredicateExpr> {
    let (input, expr) = delimited(ws(char('[')), expr, ws(char(']')))(input)?;
    Ok((input, PredicateExpr { expr }))
}

fn primary_expr(input: &str) -> IResult<&str, PrimaryExpr> {
    alt((
        literal,
        var_ref,
        map(parenthesized_expr, |e| {
            PrimaryExpr::Parenthesized(Box::new(e))
        }),
        context_item_expr,
        function_call,
    ))(input)
}

fn literal(input: &str) -> IResult<&str, PrimaryExpr> {
    map(alt((numeric_literal, string_literal)), |lit| {
        PrimaryExpr::Literal(lit)
    })(input)
}

fn numeric_literal(input: &str) -> IResult<&str, Literal> {
    alt((decimal_literal, integer_literal))(input)
}

fn var_ref(input: &str) -> IResult<&str, PrimaryExpr> {
    let (input, _) = char('$')(input)?;
    let (input, name) = qname(input)?;
    Ok((input, PrimaryExpr::Variable(name)))
}

fn parenthesized_expr(input: &str) -> IResult<&str, Expr> {
    delimited(ws(char('(')), expr, ws(char(')')))(input)
}

fn context_item_expr(input: &str) -> IResult<&str, PrimaryExpr> {
    map(char('.'), |_| PrimaryExpr::ContextItem)(input)
}

fn function_call(input: &str) -> IResult<&str, PrimaryExpr> {
    let (input, name) = qname(input)?;
    let (input, args) = delimited(
        ws(char('(')),
        separated_list0(ws(char(',')), expr_single),
        ws(char(')')),
    )(input)?;

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
    alt((pi_test, comment_test, text_test, any_kind_test))(input)
}

fn any_kind_test(input: &str) -> IResult<&str, KindTest> {
    map(tuple((tag("node"), ws(char('(')), ws(char(')')))), |_| {
        KindTest::Node
    })(input)
}

fn text_test(input: &str) -> IResult<&str, KindTest> {
    map(tuple((tag("text"), ws(char('(')), ws(char(')')))), |_| {
        KindTest::Text
    })(input)
}

fn comment_test(input: &str) -> IResult<&str, KindTest> {
    map(
        tuple((tag("comment"), ws(char('(')), ws(char(')')))),
        |_| KindTest::Comment,
    )(input)
}

fn pi_test(input: &str) -> IResult<&str, KindTest> {
    map(
        tuple((
            tag("processing-instruction"),
            ws(char('(')),
            opt(ws(string_literal)),
            ws(char(')')),
        )),
        |(_, _, literal, _)| {
            KindTest::PI(literal.map(|l| match l {
                Literal::String(s) => s,
                _ => unreachable!(),
            }))
        },
    )(input)
}

fn ws<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(multispace0, inner, multispace0)
}

fn integer_literal(input: &str) -> IResult<&str, Literal> {
    map(recognize(tuple((opt(char('-')), digit1))), |s: &str| {
        Literal::Numeric(NumericLiteral::Integer(s.parse().unwrap()))
    })(input)
}

fn decimal_literal(input: &str) -> IResult<&str, Literal> {
    map(
        recognize(tuple((opt(char('-')), opt(digit1), char('.'), digit1))),
        |s: &str| Literal::Numeric(NumericLiteral::Decimal(s.parse().unwrap())),
    )(input)
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
    ))(input)
}

// QName parser
fn qname(input: &str) -> IResult<&str, QName> {
    let (input, prefix) = opt(tuple((ncname, char(':'))))(input)?;
    let (input, local) = ncname(input)?;

    Ok((
        input,
        QName {
            prefix: prefix.map(|(p, _)| p.to_string()),
            local_part: local.to_string(),
        },
    ))
}

// NCName parser
fn ncname(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alpha1,
        many0(alt((alphanumeric1, tag("-"), tag("_")))),
    ))(input)
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
                    assert_eq!(result, expected);
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
                            predicates: vec![PredicateExpr {
                                expr: Expr::Path(PathExpr {
                                    is_absolute: false,
                                    is_descendant: false,
                                    steps: vec![StepExpr::Filter(FilterExpr {
                                        primary: PrimaryExpr::Literal(Literal::Numeric(
                                            NumericLiteral::Integer(2),
                                        )),
                                        predicates: PredicateListExpr { predicates: vec![] },
                                    })],
                                }),
                            }],
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
                    assert_eq!(result, expected);
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
                        axis: Axis::Child,
                        node_test: NodeTest::Wildcard,
                        predicates: PredicateListExpr {
                            predicates: vec![PredicateExpr {
                                expr: Expr::Path(PathExpr {
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
                                }),
                            }],
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
                            axis: Axis::Child,
                            node_test: NodeTest::Name(QName {
                                prefix: None,
                                local_part: "div".to_string(),
                            }),
                            predicates: PredicateListExpr {
                                predicates: vec![PredicateExpr {
                                    expr: Expr::Relational(
                                        Box::new(Expr::Path(PathExpr {
                                            is_absolute: false,
                                            is_descendant: false,
                                            steps: vec![StepExpr::Filter(FilterExpr {
                                                primary: PrimaryExpr::Function(
                                                    CoreFunction::Position,
                                                ),
                                                predicates: PredicateListExpr {
                                                    predicates: vec![],
                                                },
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
                                                predicates: PredicateListExpr {
                                                    predicates: vec![],
                                                },
                                            })],
                                        })),
                                    ),
                                }],
                            },
                        }),
                        StepExpr::Axis(AxisStep {
                            axis: Axis::Child,
                            node_test: NodeTest::Wildcard,
                            predicates: PredicateListExpr {
                                predicates: vec![PredicateExpr {
                                    expr: Expr::Path(PathExpr {
                                        is_absolute: false,
                                        is_descendant: false,
                                        steps: vec![StepExpr::Filter(FilterExpr {
                                            primary: PrimaryExpr::Function(CoreFunction::Last),
                                            predicates: PredicateListExpr { predicates: vec![] },
                                        })],
                                    }),
                                }],
                            },
                        }),
                    ],
                }),
            ),
        ];

        for (input, expected) in cases {
            match parse(input) {
                Ok(result) => {
                    assert_eq!(result, expected);
                },
                Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
            }
        }
    }
}
