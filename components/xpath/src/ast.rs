/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, BinaryOperator, Box<Expr>),
    Negate(Box<Expr>),
    Path(PathExpr),
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum BinaryOperator {
    Or,
    And,
    Union,
    /// `=`
    Equal,
    /// `!=`
    NotEqual,
    /// `<`
    LessThan,
    /// `>`
    GreaterThan,
    /// `<=`
    LessThanOrEqual,
    /// `>=`
    GreaterThanOrEqual,
    /// `+`
    Add,
    /// `-`
    Subtract,
    /// `*`
    Multiply,
    /// `div`
    Divide,
    /// `mod`
    Modulo,
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
    Integer(i64),
    Decimal(f64),
    String(String),
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
