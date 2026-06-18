//! Our own intermediate representation of CDDL.
//! This can only handle a subset of CDDL which is used in WebDriver BiDi spec.
//!
//! Some special cases:
//!
//! - `script.LocalValue`: should flatten `script.DataLocalValue` if only one.

use std::borrow::Cow;

type CowStr<'a> = Cow<'a, str>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct File<'a> {
    rules: Vec<Rule<'a>>,
}

/// A rule definition in CDDL, with the syntax `Name = Type`.
///
/// We do not distinguish between type rule and group rule, because
/// they have no difference for Rust.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rule<'a> {
    pub(crate) name: Name<'a>,
    pub(crate) ty: Type<'a>,
}

/// A parsed representation of a name, categorized by its source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Name<'a> {
    /// A primitive type name.
    ///
    /// Examples include `any` and `text`. See [`Primitive`] for details.
    Primitive(Primitive),
    /// A top-level module symbol name.
    ///
    /// This represents a name without a dot (`.`), located in the
    /// root module of a file. Examples include `Command` and `CommandData`.
    ///
    /// This field preserves the original case.
    Global(CowStr<'a>),
    /// A qualified name prefixed by its module name.
    ///
    /// Example include `session.End` and `input.Origin`.
    Prefixed {
        /// The full, original name str.
        raw: CowStr<'a>,
        /// The module prefix.
        prefix: CowStr<'a>,
        /// The stem name without prefix.
        name: CowStr<'a>,
    },
}

/// Primitive types defined in CDDL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Primitive {
    /// The `any` type, mapped to [`serde_json::Value`].
    Any,
    /// The `text` type, mapped to [`String`].
    Text,
    /// The `float` type, mapped to [`f64`].
    /// This also covers `number` and float ranges (e.g. `0.1..2.0`).
    Float,
    /// The int type, mapped to [`i64`].
    /// This also covers int ranges (e.g. `-9007199254740991..9007199254740991`).
    Int,
    /// The uint type, mapped to [`u64`].
    /// This also covers uint ranges (e.g. `0..9007199254740991`).
    Uint,
    /// The `bool` type, mapped to [`bool`].
    /// This also covers `true` and `false`.
    Bool,
    /// The `null` type.
    /// Though we have this type. This should be handled by [`Type::Optional`]
    /// variant, and this is only for parsing purpose.
    Null,
}

impl<'a> Name<'a> {
    /// Parse a raw str into a name.
    pub(crate) fn parse(raw: &'a str) -> Self {
        let splits: Vec<_> = raw.split(".").collect();
        match splits[..] {
            [global] => match global {
                "any" => Self::Primitive(Primitive::Any),
                "text" => Self::Primitive(Primitive::Text),
                "float" | "number" => Self::Primitive(Primitive::Float),
                "int" => Self::Primitive(Primitive::Int),
                "uint" => Self::Primitive(Primitive::Uint),
                "bool" | "true" | "false" => Self::Primitive(Primitive::Bool),
                "null" => Self::Primitive(Primitive::Null),
                _ => Self::Global(global.into()),
            },
            [prefix, name] => Self::Prefixed {
                raw: raw.into(),
                prefix: prefix.into(),
                name: name.into(),
            },
            _ => unreachable!("WebDriver BiDi should have at most one module level"),
        }
    }
}

impl<'a> From<&'a str> for Name<'a> {
    fn from(value: &'a str) -> Self {
        Self::parse(value)
    }
}

/// A recursively defined type, used in right-hand side of a [`Rule`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type<'a> {
    /// A single alias, such as `text`, `0.1..2.0`, or `network.StringValue`.
    ///
    /// See [`Name`] for details on naming variants.
    Named(Name<'a>),
    /// An array containing items of same type.
    ///
    /// This is typically mapped to a Rust [`Vec`].
    ///
    /// # Example
    ///
    /// ```text
    /// script.ListLocalValue = [*script.LocalValue]
    /// ```
    Array(Box<Type<'a>>),
    /// An array with fixed length and type.
    ///
    /// This is typically mapped to a Rust tuple `(A, B, ..)`.
    ///
    /// # Example
    ///
    /// ```text
    /// script.MappingLocalValue = [*[(script.LocalValue / text), script.LocalValue]]
    /// ```
    /// This example is an `Array` of `Tuple`.
    Tuple(Vec<Type<'a>>),
    /// A map composed of key-value fields or shared groups.
    ///
    /// The map is typicall mapped to a Rust `struct`.
    ///
    /// See [`Field`] for details on field variants.
    ///
    /// # Example
    ///
    /// Ordinary top-level map:
    ///
    /// ```text
    /// Command = {
    ///   id: js-uint,
    ///   CommandData,
    ///   Extensible,
    /// }
    /// ```
    ///
    /// A map can be nested inline:
    ///
    /// ```text
    /// session.NewResult = {
    ///   sessionId: text,
    ///   capabilities: {
    ///     acceptInsecureCerts: bool,
    ///     browserName: text,
    ///     browserVersion: text,
    ///     platformName: text,
    ///     setWindowRect: bool,
    ///     userAgent: text,
    ///     ? proxy: session.ProxyConfiguration,
    ///     ? unhandledPromptBehavior: session.UserPromptHandler,
    ///     ? webSocketUrl: text,
    ///     Extensible
    ///   }
    /// }
    /// ```
    Map(Vec<Field<'a>>),
    /// One or multiple literals.
    ///
    /// # Example
    ///
    /// ```text
    /// script.DateLocalValue = (
    ///   type: "date",  ; single literal
    ///   value: text
    /// )
    ///
    /// network.SetCacheBehaviorParameters = {
    ///   cacheBehavior: "default" / "bypass", ; multiple literal
    ///   ? contexts: [+browsingContext.BrowsingContext]
    /// }
    /// ```
    ///
    /// `Literals` is actually a special case of `Choices`,
    /// but we want to handle it separately as they can be
    /// field for enum.
    Literals(Vec<CowStr<'a>>),
    /// A choice with a `null` variant. e.g.
    ///
    /// ```text
    /// log.BaseLogEntry = (
    ///   level: log.Level,
    ///   source: script.Source,
    ///   text: text / null,       ; this field
    ///   timestamp: js-uint,
    ///   ? stackTrace: script.StackTrace,
    /// )
    /// ```
    ///
    /// `Optional` is actually a special case of `Choices`
    Optional(Box<Type<'a>>),
    /// A choice that is not `Literals` or `Optional`.
    ///
    /// This is typically mapped to a Rust [`enum`].
    ///
    /// # Example
    ///
    /// ```text
    /// BrowserCommand = (
    ///   browser.Close //
    ///   browser.CreateUserContext //
    ///   browser.GetClientWindows //
    ///   browser.GetUserContexts //
    ///   browser.RemoveUserContext //
    ///   browser.SetClientWindowState //
    ///   browser.SetDownloadBehavior
    /// )
    ///
    /// BrowserResult = (
    ///   browser.CloseResult /
    ///   browser.CreateUserContextResult /
    ///   browser.GetClientWindowsResult /
    ///   browser.GetUserContextsResult /
    ///   browser.RemoveUserContextResult /
    ///   browser.SetClientWindowStateResult /
    ///   browser.SetDownloadBehaviorResult
    /// )
    /// ```
    Choices(Vec<Type<'a>>),
    /// A mapping defined by the `=>` operator.
    ///
    /// This typically mapped to Rust [`HashMap`].
    ///
    /// # Example
    ///
    /// ```text
    /// Extensible = (*text => any)
    /// ```
    Arrow(Box<Type<'a>>, Box<Type<'a>>),
}

/// Represent a map's field in CDDL definition.
///
/// A field can either be a named field or an anonymous embedded type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Field<'a> {
    /// A bareword named field with key and value type.
    ///
    /// # Example
    ///
    /// ```text
    /// session.CapabilitiesRequest = {
    ///   ? alwaysMatch: session.CapabilityRequest,
    ///   ? firstMatch: [*session.CapabilityRequest]
    /// }
    /// ```
    Keyed {
        // The question mark.
        skip: bool,
        key: CowStr<'a>,
        ty: Type<'a>,
    },
    /// A field of embedded group or groups.
    ///
    /// # Examples
    ///
    /// Single group name
    ///
    /// ```text
    /// Command = {
    ///   id: js-uint,
    ///   CommandData, ; group
    ///   Extensible, ; group
    /// }
    /// ```
    Group(Name<'a>),
    ///
    /// In rare cases the field can be inline multiple and nested:
    ///
    /// # Examples
    ///
    /// ```text
    /// browser.DownloadBehavior = {
    ///   (
    ///     browser.DownloadBehaviorAllowed //
    ///     browser.DownloadBehaviorDenied
    ///   )
    /// }
    ///
    /// network.ContinueWithAuthParameters = {
    ///   request: network.Request,
    ///   (network.ContinueWithAuthCredentials // network.ContinueWithAuthNoCredentials)
    /// }
    ///
    /// emulation.SetGeolocationOverrideParameters = {
    ///   (
    ///     (coordinates: emulation.GeolocationCoordinates / null) //
    ///     (error: emulation.GeolocationPositionError)
    ///   ),
    ///   ? contexts: [+browsingContext.BrowsingContext],
    ///   ? userContexts: [+browser.UserContext],
    /// }
    ///
    /// session.NewResult = {
    ///   sessionId: text,
    ///   capabilities: {
    ///     acceptInsecureCerts: bool,
    ///     browserName: text,
    ///     browserVersion: text,
    ///     platformName: text,
    ///     setWindowRect: bool,
    ///     userAgent: text,
    ///     ? proxy: session.ProxyConfiguration,
    ///     ? unhandledPromptBehavior: session.UserPromptHandler,
    ///     ? webSocketUrl: text,
    ///     Extensible
    ///   }
    /// }
    /// ```
    Inline(Type<'a>),
}

/// The trait to visit [`File`].
pub trait Visitor<'a> {
    fn visit_file(&mut self, file: &mut File<'a>) {
        walk_file(self, file);
    }
    fn visit_rule(&mut self, rule: &mut Rule<'a>) {
        walk_rule(self, rule);
    }
    fn visit_name(&mut self, _name: &mut Name<'a>) {}
    fn visit_ty(&mut self, ty: &mut Type<'a>) {
        walk_ty(self, ty);
    }
    fn visit_field(&mut self, field: &mut Field<'a>) {
        walk_field(self, field);
    }
}

pub fn walk_file<'a, V: Visitor<'a> + ?Sized>(visitor: &mut V, file: &mut File<'a>) {
    for rule in &mut file.rules {
        visitor.visit_rule(rule);
    }
}

pub fn walk_rule<'a, V: Visitor<'a> + ?Sized>(visitor: &mut V, rule: &mut Rule<'a>) {
    visitor.visit_name(&mut rule.name);
    visitor.visit_ty(&mut rule.ty);
}

pub fn walk_ty<'a, V: Visitor<'a> + ?Sized>(visitor: &mut V, ty: &mut Type<'a>) {
    match ty {
        Type::Named(name) => visitor.visit_name(name),
        Type::Array(inner) | Type::Optional(inner) => visitor.visit_ty(inner),
        Type::Tuple(tys) | Type::Choices(tys) => {
            for ty in tys {
                visitor.visit_ty(ty);
            }
        },
        Type::Map(fields) => {
            for field in fields {
                visitor.visit_field(field);
            }
        },
        Type::Arrow(key, value) => {
            visitor.visit_ty(key);
            visitor.visit_ty(value);
        },
        Type::Literals(_) => {},
    }
}

pub fn walk_field<'a, V: Visitor<'a> + ?Sized>(visitor: &mut V, field: &mut Field<'a>) {
    match field {
        Field::Keyed { ty, .. } => visitor.visit_ty(ty),
        Field::Group(name) => visitor.visit_name(name),
        Field::Inline(inner) => visitor.visit_ty(inner),
    }
}
