/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// EBNF grammar of XPath 1.0
// [1]  XPath              ::= Expr
// [2]  Expr               ::= ExprSingle
// [3]  ExprSingle         ::= OrExpr
// [4]  OrExpr             ::= AndExpr ( "or" AndExpr )*
// [5]  AndExpr            ::= EqualityExpr ( "and" EqualityExpr )*
// [6]  EqualityExpr       ::= RelationalExpr ( ("=" \| "!=") // RelationalExpr )*
// [7]  RelationalExpr     ::= AdditiveExpr ( ("<" \| ">" \| "<=" \| ">=") AdditiveExpr )*
// [8]  AdditiveExpr       ::= MultiplicativeExpr ( ("+" \| "-") MultiplicativeExpr )*
// [9]  MultiplicativeExpr ::= UnaryExpr ( ( "*" \| "div" \| "mod") UnaryExpr )*
// [10] UnaryExpr          ::= "-"* UnionExpr
// [11] UnionExpr          ::= PathExpr ( "|" PathExpr )*
// [12] PathExpr           ::= ("/" RelativePathExpr?) \| ("//" RelativePathExpr) \| RelativePathExpr
// [13] RelativePathExpr   ::= StepExpr (("/" \| "//") StepExpr)*
// [14] StepExpr           ::= FilterExpr \| AxisStep
// [15] AxisStep           ::= (ReverseStep \| ForwardStep) PredicateList
// [16] ForwardStep        ::= (ForwardAxis NodeTest) \| AbbrevForwardStep
// [17] ForwardAxis        ::= ("child" "::")
//                               \| ("descendant" "::")
//                               \| ("attribute" "::")
//                               \| ("self" "::")
//                               \| ("descendant-or-self" "::")
//                               \| ("following-sibling" "::")
//                               \| ("following" "::")
//                               \| ("namespace" "::")
// [18] AbbrevForwardStep  ::= '@'? NodeTest
// [19] ReverseStep        ::= (ReverseAxis NodeTest) \| AbbrevReverseStep
// [20] ReverseAxis        ::= ("parent" "::")
//                               \| ("ancestor" "::")
//                               \| ("preceding-sibling" "::")
//                               \| ("preceding" "::")
//                               \| ("ancestor-or-self" "::")
// [21] AbbrevReverseStep  ::= ”..” /* xgc: predicate */
// [22] NodeTest           ::= KindTest \| NameTest
// [23] NameTest           ::= QName \| Wildcard
// [24] Wildcard           ::= "*" \| (NCName ":" "*")
// [25] FilterExpr         ::= PrimaryExpr PredicateList
// [26] PredicateList      ::= Predicate*
// [27] Predicate          ::= "[" Expr "]"
// [28] PrimaryExpr        ::= Literal \| VarRef \| ParenthesizedExpr \| ContextItemExpr \| FunctionCall
// [29] Literal            ::= NumericLiteral \| StringLiteral
// [30] NumericLiteral     ::= IntegerLiteral \| DecimalLiteral
// [31] VarRef             ::= "$" VarName
// [32] VarName            ::= QName
// [33] ParenthesizedExpr  ::= "(" Expr ")"
// [34] ContextItemExpr    ::= "."
// [35] FunctionCall       ::= QName "(" ( ExprSingle ( "," ExprSingle )* )? ')'
// [36] KindTest           ::= PITest \| CommentTest \| TextTest \| AnyKindTest
// [37] AnyKindTest        ::= "node" "(" ")"
// [38] TextTest           ::= "text" "(" ")"
// [39] CommentTest        ::= "comment" "(" ")"
// [40] PITest             ::= "processing-instruction" "(" StringLiteral? ")"
// [41] StringLiteral      ::= '"' [^"]* '"' | "'" [^']* "'"

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{alpha1, alphanumeric1, char, digit1, multispace0};
use nom::combinator::{map, opt, recognize, value};
use nom::error::{Error as NomError, ErrorKind as NomErrorKind};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::{Finish, IResult};

pub fn parse(input: &str) -> Result<Expr, OwnedParserError> {
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
    pub is_absolute: bool,
    pub is_descendant: bool,
    pub steps: Vec<StepExpr>,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct PredicateListExpr {
    pub predicates: Vec<PredicateExpr>,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct PredicateExpr {
    pub expr: Expr,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct FilterExpr {
    pub primary: PrimaryExpr,
    pub predicates: PredicateListExpr,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum StepExpr {
    Filter(FilterExpr),
    Axis(AxisStep),
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct AxisStep {
    pub axis: Axis,
    pub node_test: NodeTest,
    pub predicates: PredicateListExpr,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum Axis {
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
pub enum NodeTest {
    Name(QName),
    Wildcard,
    Kind(KindTest),
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct QName {
    pub prefix: Option<String>,
    pub local_part: String,
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
pub enum KindTest {
    PI(Option<String>),
    Comment,
    Text,
    Node,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum PrimaryExpr {
    Literal(Literal),
    Variable(QName),
    Parenthesized(Box<Expr>),
    ContextItem,
    /// We only support the built-in core functions
    Function(CoreFunction),
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum Literal {
    Numeric(NumericLiteral),
    String(String),
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum NumericLiteral {
    Integer(u64),
    Decimal(f64),
}

/// In the DOM we do not support custom functions, so we can enumerate the usable ones
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum CoreFunction {
    // Node Set Functions
    Last,                            // last()
    Position,                        // position()
    Count(Box<Expr>),                // count(node-set)
    Id(Box<Expr>),                   // id(object)
    LocalName(Option<Box<Expr>>),    // local-name(node-set?)
    NamespaceUri(Option<Box<Expr>>), // namespace-uri(node-set?)
    Name(Option<Box<Expr>>),         // name(node-set?)

    // String Functions
    String(Option<Box<Expr>>),                          // string(object?)
    Concat(Vec<Expr>),                                  // concat(string, string, ...)
    StartsWith(Box<Expr>, Box<Expr>),                   // starts-with(string, string)
    Contains(Box<Expr>, Box<Expr>),                     // contains(string, string)
    SubstringBefore(Box<Expr>, Box<Expr>),              // substring-before(string, string)
    SubstringAfter(Box<Expr>, Box<Expr>),               // substring-after(string, string)
    Substring(Box<Expr>, Box<Expr>, Option<Box<Expr>>), // substring(string, number, number?)
    StringLength(Option<Box<Expr>>),                    // string-length(string?)
    NormalizeSpace(Option<Box<Expr>>),                  // normalize-space(string?)
    Translate(Box<Expr>, Box<Expr>, Box<Expr>),         // translate(string, string, string)

    // Number Functions
    Number(Option<Box<Expr>>), // number(object?)
    Sum(Box<Expr>),            // sum(node-set)
    Floor(Box<Expr>),          // floor(number)
    Ceiling(Box<Expr>),        // ceiling(number)
    Round(Box<Expr>),          // round(number)

    // Boolean Functions
    Boolean(Box<Expr>), // boolean(object)
    Not(Box<Expr>),     // not(boolean)
    True,               // true()
    False,              // false()
    Lang(Box<Expr>),    // lang(string)
}

impl CoreFunction {
    pub fn name(&self) -> &'static str {
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

    pub fn min_args(&self) -> usize {
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

    pub fn max_args(&self) -> Option<usize> {
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
    pub fn is_valid_arity(&self, num_args: usize) -> bool {
        let min = self.min_args();
        let max = self.max_args();

        num_args >= min && max.map_or(true, |max| num_args <= max)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OwnedParserError {
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

// [1] XPath and
// [2] Expr combined.
/// Top-level parser
fn expr(input: &str) -> IResult<&str, Expr> {
    expr_single(input)
}

// [3] ExprSingle
fn expr_single(input: &str) -> IResult<&str, Expr> {
    or_expr(input)
}

// [4] OrExpr
fn or_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = and_expr(input)?;
    let (input, rest) = many0(preceded(ws(tag("or")), and_expr))(input)?;

    Ok((
        input,
        rest.into_iter()
            .fold(first, |acc, expr| Expr::Or(Box::new(acc), Box::new(expr))),
    ))
}

// [5] AndExpr
fn and_expr(input: &str) -> IResult<&str, Expr> {
    let (input, first) = equality_expr(input)?;
    let (input, rest) = many0(preceded(ws(tag("and")), equality_expr))(input)?;

    Ok((
        input,
        rest.into_iter()
            .fold(first, |acc, expr| Expr::And(Box::new(acc), Box::new(expr))),
    ))
}

// [6] EqualityExpr
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

// [7] RelationalExpr
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

// [8] AdditiveExpr
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

// [9] MultiplicativeExpr
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

// [10] UnaryExpr
fn unary_expr(input: &str) -> IResult<&str, Expr> {
    let (input, minus_count) = many0(ws(char('-')))(input)?;
    let (input, expr) = union_expr(input)?;

    Ok((
        input,
        (0..minus_count.len()).fold(expr, |acc, _| Expr::Unary(UnaryOp::Minus, Box::new(acc))),
    ))
}

// [11] UnionExpr
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

// [12] PathExpr
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

// [13] RelativePathExpr
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

// [14] StepExpr
fn step_expr(input: &str) -> IResult<&str, StepExpr> {
    alt((
        map(filter_expr, StepExpr::Filter),
        map(axis_step, StepExpr::Axis),
    ))(input)
}

// [15] AxisStep
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

// [16] ForwardStep
fn forward_step(input: &str) -> IResult<&str, (Axis, NodeTest)> {
    alt((
        // ForwardAxis NodeTest
        pair(forward_axis, node_test),
        // AbbrevForwardStep
        abbrev_forward_step,
    ))(input)
}

// [17] ForwardAxis
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

// [18] AbbrevForwardStep
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

// [19] ReverseStep
fn reverse_step(input: &str) -> IResult<&str, (Axis, NodeTest)> {
    alt((
        // ReverseAxis NodeTest
        pair(reverse_axis, node_test),
        // AbbrevReverseStep
        abbrev_reverse_step,
    ))(input)
}

// [20] ReverseAxis
fn reverse_axis(input: &str) -> IResult<&str, Axis> {
    alt((
        value(Axis::Parent, tag("parent::")),
        value(Axis::Ancestor, tag("ancestor::")),
        value(Axis::PrecedingSibling, tag("preceding-sibling::")),
        value(Axis::Preceding, tag("preceding::")),
        value(Axis::AncestorOrSelf, tag("ancestor-or-self::")),
    ))(input)
}

// [21] AbbrevReverseStep
fn abbrev_reverse_step(input: &str) -> IResult<&str, (Axis, NodeTest)> {
    map(tag(".."), |_| {
        (Axis::Parent, NodeTest::Kind(KindTest::Node))
    })(input)
}

// [22] NodeTest
fn node_test(input: &str) -> IResult<&str, NodeTest> {
    alt((
        map(kind_test, NodeTest::Kind),
        map(name_test, |name| match name {
            NameTest::Wildcard => NodeTest::Wildcard,
            NameTest::QName(qname) => NodeTest::Name(qname),
        }),
    ))(input)
}

// [23] NameTest and
// [24] Wildcard combined
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

// [25] FilterExpr
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

// [26] PredicateList
fn predicate_list(input: &str) -> IResult<&str, PredicateListExpr> {
    let (input, predicates) = many0(predicate)(input)?;
    Ok((input, PredicateListExpr { predicates }))
}

// [27] Predicate
fn predicate(input: &str) -> IResult<&str, PredicateExpr> {
    let (input, expr) = delimited(ws(char('[')), expr, ws(char(']')))(input)?;
    Ok((input, PredicateExpr { expr }))
}

// [28] PrimaryExpr
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

// [29] Literal
fn literal(input: &str) -> IResult<&str, PrimaryExpr> {
    map(alt((numeric_literal, string_literal)), |lit| {
        PrimaryExpr::Literal(lit)
    })(input)
}

// [30] NumericLiteral
fn numeric_literal(input: &str) -> IResult<&str, Literal> {
    alt((decimal_literal, integer_literal))(input)
}

// [31] VarRef and
// [32] VarName
fn var_ref(input: &str) -> IResult<&str, PrimaryExpr> {
    let (input, _) = char('$')(input)?;
    let (input, name) = qname(input)?;
    Ok((input, PrimaryExpr::Variable(name)))
}

// [33] ParenthesizedExpr
fn parenthesized_expr(input: &str) -> IResult<&str, Expr> {
    delimited(ws(char('(')), expr, ws(char(')')))(input)
}

// [34] ContextItemExpr
fn context_item_expr(input: &str) -> IResult<&str, PrimaryExpr> {
    map(char('.'), |_| PrimaryExpr::ContextItem)(input)
}

// [35] FunctionCall
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

// [36] KindTest
fn kind_test(input: &str) -> IResult<&str, KindTest> {
    alt((pi_test, comment_test, text_test, any_kind_test))(input)
}

// [37] AnyKindTest
fn any_kind_test(input: &str) -> IResult<&str, KindTest> {
    map(tuple((tag("node"), ws(char('(')), ws(char(')')))), |_| {
        KindTest::Node
    })(input)
}

// [38] TextTest
fn text_test(input: &str) -> IResult<&str, KindTest> {
    map(tuple((tag("text"), ws(char('(')), ws(char(')')))), |_| {
        KindTest::Text
    })(input)
}

// [39] CommentTest
fn comment_test(input: &str) -> IResult<&str, KindTest> {
    map(
        tuple((tag("comment"), ws(char('(')), ws(char(')')))),
        |_| KindTest::Comment,
    )(input)
}

// [40] PITest
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

// [41] StringLiteral
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
    use insta::{assert_debug_snapshot, with_settings};
    use rstest::rstest;

    use super::*;

    macro_rules! set_snapshot_suffix {
        ($($expr:expr),*) => {
            let mut settings = insta::Settings::clone_current();
            settings.set_snapshot_suffix(format!($($expr,)*));
            let _guard = settings.bind_to_scope();
        }
    }

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

    #[rstest]
    #[case("position", "position()")]
    #[case("last", "last()")]
    #[case("concat", "concat('hello', ' ', 'world')")]
    #[case("node", "node()[1]")]
    #[case("text", "text()[contains(., 'test')]")]
    #[case("pi", "processing-instruction('test')[2]")]
    fn test_filter_expr(#[case] name: &str, #[case] input: &str) {
        set_snapshot_suffix!("{}", name);

        if let Ok((_, output)) = expr(input) {
            with_settings!({ description => input }, {
                assert_debug_snapshot!(output);
            });
        } else {
            panic!("Failed to parse '{}'", input);
        }
    }

    #[rstest]
    #[case("contains_class", "//*[contains(@class, 'test')]")]
    #[case("position_last", "//div[position() > 1]/*[last()]")]
    #[case("following_sibling", "ancestor::*[1]/following-sibling::div[1]")]
    #[case("div_or_span", "//*[self::div or self::span][@class]")]
    #[case(
        "pi_string_length",
        "//processing-instruction('test')[string-length(.) > 0]"
    )]
    fn test_complex_paths(#[case] name: &str, #[case] input: &str) {
        set_snapshot_suffix!("{}", name);

        if let Ok((_, output)) = expr(input) {
            with_settings!({ description => input }, {
                assert_debug_snapshot!(output);
            });
        } else {
            panic!("Failed to parse '{}'", input);
        }
    }
}
