//! Our own intermediate representation of CDDL.
//! This can only handle a subset of CDDL which is used in WebDriver BiDi spec.
//!
//! Some special cases:
//!
//! - `script.LocalValue`: should flatten `script.DataLocalValue` if only one.

use std::borrow::Cow;

type CowStr<'a> = Cow<'a, str>;

/// A rule definition in CDDL, with the syntax `Name = Type`.
///
/// We do not distinguish between type rule and group rule, because
/// they have no difference for Rust.
pub struct Rule<'a> {
    name: Name<'a>,
    ty: Type<'a>,
}

/// A parsed representation of a name, categorized by its source.
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
        /// This field preserves the original case.
        raw: CowStr<'a>,
        /// The module prefix.
        /// This field is converted to snake_case.
        prefix: CowStr<'a>,
        /// The stem name without prefix.
        /// This field preserves the original case.
        name: CowStr<'a>,
    },
}

/// Primitive types defined in CDDL.
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
}

/// A recursively defined type, used in right-hand side of a [`Rule`].
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
    /// ```text
    /// script.ListLocalValue = [*script.LocalValue]
    /// ```
    Array(Box<Type<'a>>),
    /// An array with fixed length and type.
    ///
    /// This is typically mapped to a Rust tuple `(A, B, ..)`.
    ///
    /// # Example
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
pub enum Field<'a> {
    /// A bareword named field with name and value type.
    ///
    /// # Example
    /// ```cddl
    /// Command = {
    ///   id: js-uint,
    ///   CommandData,
    ///   Extensible,
    /// }
    /// ```
    Named {
        // The question mark.
        skip: bool,
        name: CowStr<'a>,
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
    ///   CommandData, ; group name
    ///   Extensible,  ; group name
    /// }
    /// ```
    ///
    /// In rare cases the group can be inline multiple:
    ///
    /// ```text
    /// browser.DownloadBehavior = {
    ///   (
    ///     browser.DownloadBehaviorAllowed //
    ///     browser.DownloadBehaviorDenied
    ///   )
    /// }
    /// ```
    Group(Vec<Type<'a>>),
}
