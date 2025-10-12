/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum Expression {
    Binary(Box<Expression>, BinaryOperator, Box<Expression>),
    Negate(Box<Expression>),
    Path(PathExpression),
    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#section-Location-Steps>
    LocationStep(LocationStepExpression),
    Filter(FilterExpression),
    Literal(Literal),
    Variable(QName),
    ContextItem,
    /// We only support the built-in core functions.
    Function(CoreFunction),
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
pub struct PathExpression {
    /// Whether this is an absolute (as opposed to a relative) path expression.
    ///
    /// Absolute paths always start at the starting node, not the context node.
    pub(crate) is_absolute: bool,
    /// Whether this expression starts with `//`. If it does, then an implicit
    /// `descendant-or-self::node()` step will be added.
    pub(crate) has_implicit_descendant_or_self_step: bool,
    pub(crate) steps: Vec<Expression>,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct PredicateListExpression {
    pub(crate) predicates: Vec<Expression>,
}

#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct FilterExpression {
    pub(crate) expression: Box<Expression>,
    pub(crate) predicates: PredicateListExpression,
}

/// <https://www.w3.org/TR/1999/REC-xpath-19991116/#section-Location-Steps>
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct LocationStepExpression {
    pub(crate) axis: Axis,
    pub(crate) node_test: NodeTest,
    pub(crate) predicate_list: PredicateListExpression,
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
pub enum Literal {
    Integer(i64),
    Decimal(f64),
    String(String),
}

/// In the DOM we do not support custom functions, so we can enumerate the usable ones
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub enum CoreFunction {
    // Node Set Functions
    /// last()
    Last,
    /// position()
    Position,
    /// count(node-set)
    Count(Box<Expression>),
    /// id(object)
    Id(Box<Expression>),
    /// local-name(node-set?)
    LocalName(Option<Box<Expression>>),
    /// namespace-uri(node-set?)
    NamespaceUri(Option<Box<Expression>>),
    /// name(node-set?)
    Name(Option<Box<Expression>>),

    // String Functions
    /// string(object?)
    String(Option<Box<Expression>>),
    /// concat(string, string, ...)
    Concat(Vec<Expression>),
    /// starts-with(string, string)
    StartsWith(Box<Expression>, Box<Expression>),
    /// contains(string, string)
    Contains(Box<Expression>, Box<Expression>),
    /// substring-before(string, string)
    SubstringBefore(Box<Expression>, Box<Expression>),
    /// substring-after(string, string)
    SubstringAfter(Box<Expression>, Box<Expression>),
    /// substring(string, number, number?)
    Substring(Box<Expression>, Box<Expression>, Option<Box<Expression>>),
    /// string-length(string?)
    StringLength(Option<Box<Expression>>),
    /// normalize-space(string?)
    NormalizeSpace(Option<Box<Expression>>),
    /// translate(string, string, string)
    Translate(Box<Expression>, Box<Expression>, Box<Expression>),

    // Number Functions
    /// number(object?)
    Number(Option<Box<Expression>>),
    /// sum(node-set)
    Sum(Box<Expression>),
    /// floor(number)
    Floor(Box<Expression>),
    /// ceiling(number)
    Ceiling(Box<Expression>),
    /// round(number)
    Round(Box<Expression>),

    // Boolean Functions
    /// boolean(object)
    Boolean(Box<Expression>),
    /// not(boolean)
    Not(Box<Expression>),
    /// true()
    True,
    /// false()
    False,
    /// lang(string)
    Lang(Box<Expression>),
}
