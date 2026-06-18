//! A `nom`-based parser which can parse CDDL into our IR.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{is_not, tag, take_until, take_while1},
    character::complete::{alpha1, digit1, multispace1},
    combinator::{opt, recognize, value},
    multi::{many0, separated_list1},
    sequence::{delimited, preceded, separated_pair, terminated},
};

use crate::ast::{Field, Name, Primitive, Rule, Type};

/// Parse a [`Rule`].
pub fn rule<'a>(s: &'a str) -> IResult<&'a str, Rule<'a>> {
    (name, (ws, tag("="), ws), ty)
        .map(|(name, _, ty)| Rule { name, ty })
        .parse(s)
}

/// Parse a [`Name`].
fn name<'a>(s: &'a str) -> IResult<&'a str, Name<'a>> {
    alt((name_ident, name_range)).parse(s)
}

/// Parse a [`Name`] from identifier.
fn name_ident<'a>(s: &'a str) -> IResult<&'a str, Name<'a>> {
    take_while1(is_w_dot_dash)
        .map_opt(check_first_alpha)
        .map(Name::parse)
        .parse(s)
}

/// Parse a [`Name`] from number range.
fn name_range<'a>(s: &'a str) -> IResult<&'a str, Name<'a>> {
    separated_pair(literal_number, (tag(".."), ws), literal_number)
        .map(|(lo, _hi)| {
            Name::Primitive(if lo.contains(".") {
                Primitive::Float
            } else if lo.contains("-") {
                Primitive::Int
            } else {
                Primitive::Uint
            })
        })
        .parse(s)
}

/// Parse a [`Type`].
fn ty<'a>(s: &'a str) -> IResult<&'a str, Type<'a>> {
    // choices (optional, literal) do not have delimiter,
    // we first parse many, then check num.
    terminated(
        separated_list1((ws, alt((tag("//"), tag("/"))), ws), ty_atom).map(one_or_many),
        many0(ty_annotation),
    )
    .parse(s)
}

/// Parse [`Type`] without noticing a choice.
fn ty_atom<'a>(s: &'a str) -> IResult<&'a str, Type<'a>> {
    alt((
        ty_arrow,
        ty_paren,
        ty_brace,
        ty_literal,
        name.map(Type::Named),
        ty_tuple,
        ty_array,
    ))
    .parse(s)
}

/// Parse a paren-delimited content.
fn ty_paren<'a>(s: &'a str) -> IResult<&'a str, Type<'a>> {
    alt((
        // ty passthrough, paren is used as precedence rule
        delimited((tag("("), ws), ty, (ws, tag(")"))),
        // ty is used as struct (map) boundry
        delimited((tag("("), ws), ty_map_fields.map(Type::Map), (ws, tag(")"))),
    ))
    .parse(s)
}

/// Parse a [`Type::Literals`], the single literal case.
/// Multiple literals is corrected in [`one_or_many`].
fn ty_literal<'a>(s: &'a str) -> IResult<&'a str, Type<'a>> {
    literal_string
        .map(|s: &'a str| Type::Literals(vec![s.into()]))
        .parse(s)
}

/// Parse a [`Type::Array`].
fn ty_array<'a>(s: &'a str) -> IResult<&'a str, Type<'a>> {
    delimited(
        (tag("["), ws, alt((tag("*"), tag("+"))), ws),
        ty.map(Box::new),
        (ws, tag("]")),
    )
    .map(Type::Array)
    .parse(s)
}

/// Parse a [`Type::Tuple`].
fn ty_tuple<'a>(s: &'a str) -> IResult<&'a str, Type<'a>> {
    delimited(
        (tag("["), ws),
        separated_list1((ws, tag(","), ws), ty),
        (ws, tag("]")),
    )
    .map(Type::Tuple)
    .parse(s)
}

/// Parse a [`Type::Arrow`].
fn ty_arrow<'a>(s: &'a str) -> IResult<&'a str, Type<'a>> {
    let inner = || separated_pair(ty, (ws, tag("=>"), ws), ty);
    alt((
        delimited((tag("(*"), ws), inner(), (ws, tag(")"))),
        delimited((tag("{*"), ws), inner(), (ws, tag("}"))),
    ))
    .map(|(k, v)| Type::Arrow(k.into(), v.into()))
    .parse(s)
}

/// Parse a brace delimited by brace.
///
/// This is typically [`Type::Map`] but with some edge cases.
fn ty_brace<'a>(s: &'a str) -> IResult<&'a str, Type<'a>> {
    delimited((tag("{"), ws), ty_map_fields, (ws, tag("}")))
        .map(|fields| {
            // here we further check:
            // if fields have single inline, we flatten that.
            // See [`tests::test_parse_map_inline_choices_single_flatten`]
            if let [Field::Inline(ty)] = &fields[..] {
                return ty.clone();
            }
            Type::Map(fields)
        })
        .parse(s)
}

/// Parse inner fields of a [`Type::Map`].
fn ty_map_fields<'a>(s: &'a str) -> IResult<&'a str, Vec<Field<'a>>> {
    terminated(
        separated_list1((ws, tag(","), ws), field),
        (ws, opt(tag(","))),
    )
    .parse(s)
}

/// Parse a [`Field`].
fn field<'a>(s: &'a str) -> IResult<&'a str, Field<'a>> {
    alt((field_keyed, ty.map(Field::Inline))).parse(s)
}

/// Parse a [`Field::Keyed`].
fn field_keyed<'a>(s: &'a str) -> IResult<&'a str, Field<'a>> {
    (opt(tag("?")), ws, alpha1, ws, tag(":"), ws, ty, ws)
        .map(
            |(question_mark, _ws1, key, _ws2, _colon, _ws3, ty, _ws4)| Field::Keyed {
                skip: question_mark.is_some(),
                flatten: false,
                key: key.into(),
                ty,
            },
        )
        .parse(s)
}

/// Parse a type annotation like `.default 1.0`, `.ge 0.0`.
/// We parse it just to ignore it.
fn ty_annotation(s: &str) -> IResult<&str, ()> {
    value(
        (),
        (
            ws,
            tag("."),
            take_while1(char::is_alphabetic),
            multispace1,
            alt((value((), literal_string), value((), literal_number))),
        ),
    )
    .parse(s)
}

/// Skip whitespace and comments.
fn ws(s: &str) -> IResult<&str, ()> {
    value((), many0(alt((value((), multispace1), comment)))).parse(s)
}

/// Skip a comment till line end.
fn comment(s: &str) -> IResult<&str, ()> {
    value((), preceded(tag(";"), take_until("\n"))).parse(s)
}

/// Parse a literal string.
fn literal_string(s: &str) -> IResult<&str, &str> {
    delimited(tag("\""), is_not("\""), tag("\"")).parse(s)
}

/// Recognize a number literal.
fn literal_number(s: &str) -> IResult<&str, &str> {
    recognize((
        opt(alt((tag("+"), tag("-")))),
        digit1,
        opt(preceded(tag("."), digit1)),
    ))
    .parse(s)
}

/// Check if a char is `[\w.-]`.
fn is_w_dot_dash(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '.' | '-')
}

/// Check if first char of a string is alphabet.
fn check_first_alpha(s: &str) -> Option<&str> {
    s.chars()
        .next()
        .and_then(|c| c.is_alphabetic().then_some(s))
}

/// Distinguish between atom, optional, literal, or choices fallback.
fn one_or_many<'a>(mut atoms: Vec<Type<'a>>) -> Type<'a> {
    // check has null
    let null_index = atoms
        .iter()
        // here we assume at most one null in choices.
        .position(|a| matches!(a, Type::Named(Name::Primitive(Primitive::Null))));
    if let Some(null_index) = null_index {
        atoms.remove(null_index);
    }
    if atoms.len() == 0 {
        unreachable!("null only type is not allowed");
        // if that really happens in the future,
        // switch on `Primitive::Null` and `Option<()>`.
    }

    let mut ty: Type;

    // check single atom
    if atoms.len() == 1 {
        ty = atoms.pop().unwrap();
    }
    // check all literal
    else if let all_literal = atoms.iter().all(|a| matches!(a, Type::Literals(_)))
        && all_literal
    {
        // concat all literals
        ty = Type::Literals(
            atoms
                .into_iter()
                .flat_map(|t| {
                    if let Type::Literals(v) = t {
                        v
                    } else {
                        unreachable!()
                    }
                })
                .collect(),
        );
    }
    // fallback to choices
    else {
        ty = Type::Choices(atoms);
    }

    // add optional
    if null_index.is_some() {
        ty = Type::Optional(ty.into());
    }

    ty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_named() {
        let input = "Foo = js-uint";
        let expected = Ok((
            "",
            Rule {
                name: "Foo".into(),
                ty: Type::Named("js-uint".into()),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_arrow() {
        let input = "Extensible = (*text => any)";
        let expected = Ok((
            "",
            Rule {
                name: "Extensible".into(),
                ty: Type::Arrow(
                    Box::new(Type::Named("text".into())),
                    Box::new(Type::Named("any".into())),
                ),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_map() {
        let input = "Command = { id: js-uint, CommandData, Extensible }";
        let expected = Ok((
            "",
            Rule {
                name: "Command".into(),
                ty: Type::Map(vec![
                    Field::Keyed {
                        skip: false,
                        flatten: false,
                        key: "id".into(),
                        ty: Type::Named("js-uint".into()),
                    },
                    Field::Inline(Type::Named("CommandData".into())),
                    Field::Inline(Type::Named("Extensible".into())),
                ]),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_literals() {
        let input =
            r#"network.SetCacheBehaviorParameters = { cacheBehavior: "default" / "bypass" }"#;
        let expected = Ok((
            "",
            Rule {
                name: "network.SetCacheBehaviorParameters".into(),
                ty: Type::Map(vec![Field::Keyed {
                    skip: false,
                    flatten: false,
                    key: "cacheBehavior".into(),
                    ty: Type::Literals(vec!["default".into(), "bypass".into()]),
                }]),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_optional() {
        let input = "log.BaseLogEntry = { text: text / null }";
        let expected = Ok((
            "",
            Rule {
                name: "log.BaseLogEntry".into(),
                ty: Type::Map(vec![Field::Keyed {
                    skip: false,
                    flatten: false,
                    key: "text".into(),
                    ty: Type::Optional(Box::new(Type::Named("text".into()))),
                }]),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_array_tuple() {
        let input =
            "script.MappingLocalValue = [* [(script.LocalValue / text), script.LocalValue]]";
        let expected = Ok((
            "",
            Rule {
                name: "script.MappingLocalValue".into(),
                ty: Type::Array(Box::new(Type::Tuple(vec![
                    Type::Choices(vec![
                        Type::Named("script.LocalValue".into()),
                        Type::Named("text".into()),
                    ]),
                    Type::Named("script.LocalValue".into()),
                ]))),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_map_inline_choices_single_flatten() {
        let input = "browser.DownloadBehavior = {
  (
    browser.DownloadBehaviorAllowed //
    browser.DownloadBehaviorDenied
  )
}";
        let expected = Ok((
            "",
            Rule {
                name: "browser.DownloadBehavior".into(),
                ty: Type::Choices(vec![
                    Type::Named("browser.DownloadBehaviorAllowed".into()),
                    Type::Named("browser.DownloadBehaviorDenied".into()),
                ]),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_map_inline_choices_second() {
        let input = "network.ContinueWithAuthParameters = {
  request: network.Request,
  (network.ContinueWithAuthCredentials // network.ContinueWithAuthNoCredentials)
}";
        let expected = Ok((
            "",
            Rule {
                name: "network.ContinueWithAuthParameters".into(),
                ty: Type::Map(vec![
                    Field::Keyed {
                        skip: false,
                        flatten: false,
                        key: "request".into(),
                        ty: Type::Named("network.Request".into()),
                    },
                    Field::Inline(Type::Choices(vec![
                        Type::Named("network.ContinueWithAuthCredentials".into()),
                        Type::Named("network.ContinueWithAuthNoCredentials".into()),
                    ])),
                ]),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_map_inline_choices_complex() {
        let input = "emulation.SetGeolocationOverrideParameters = {
    (
      (coordinates: emulation.GeolocationCoordinates / null) //
      (error: emulation.GeolocationPositionError)
    ),
    ? contexts: [+browsingContext.BrowsingContext],
    ? userContexts: [+browser.UserContext],
  }";
        let expected = Ok((
            "",
            Rule {
                name: "emulation.SetGeolocationOverrideParameters".into(),
                ty: Type::Map(vec![
                    Field::Inline(Type::Choices(vec![
                        Type::Map(vec![Field::Keyed {
                            skip: false,
                            flatten: false,
                            key: "coordinates".into(),
                            ty: Type::Optional(Box::new(Type::Named(
                                "emulation.GeolocationCoordinates".into(),
                            ))),
                        }]),
                        Type::Map(vec![Field::Keyed {
                            skip: false,
                            flatten: false,
                            key: "error".into(),
                            ty: Type::Named("emulation.GeolocationPositionError".into()),
                        }]),
                    ])),
                    Field::Keyed {
                        skip: true,
                        flatten: false,
                        key: "contexts".into(),
                        ty: Type::Array(Box::new(Type::Named(
                            "browsingContext.BrowsingContext".into(),
                        ))),
                    },
                    Field::Keyed {
                        skip: true,
                        flatten: false,
                        key: "userContexts".into(),
                        ty: Type::Array(Box::new(Type::Named("browser.UserContext".into()))),
                    },
                ]),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_map_inline_map() {
        let input = "session.NewResult = {
  sessionId: text,
  capabilities: {
    acceptInsecureCerts: bool,
    browserName: text,
    browserVersion: text,
    platformName: text,
    setWindowRect: bool,
    userAgent: text,
    ? proxy: session.ProxyConfiguration,
    ? unhandledPromptBehavior: session.UserPromptHandler,
    ? webSocketUrl: text,
    Extensible
  }
}";
        let expected = Ok((
            "",
            Rule {
                name: "session.NewResult".into(),
                ty: Type::Map(vec![
                    Field::Keyed {
                        skip: false,
                        flatten: false,
                        key: "sessionId".into(),
                        ty: Type::Named("text".into()),
                    },
                    Field::Keyed {
                        skip: false,
                        flatten: false,
                        key: "capabilities".into(),
                        ty: Type::Map(vec![
                            Field::Keyed {
                                skip: false,
                                flatten: false,
                                key: "acceptInsecureCerts".into(),
                                ty: Type::Named("bool".into()),
                            },
                            Field::Keyed {
                                skip: false,
                                flatten: false,
                                key: "browserName".into(),
                                ty: Type::Named("text".into()),
                            },
                            Field::Keyed {
                                skip: false,
                                flatten: false,
                                key: "browserVersion".into(),
                                ty: Type::Named("text".into()),
                            },
                            Field::Keyed {
                                skip: false,
                                flatten: false,
                                key: "platformName".into(),
                                ty: Type::Named("text".into()),
                            },
                            Field::Keyed {
                                skip: false,
                                flatten: false,
                                key: "setWindowRect".into(),
                                ty: Type::Named("bool".into()),
                            },
                            Field::Keyed {
                                skip: false,
                                flatten: false,
                                key: "userAgent".into(),
                                ty: Type::Named("text".into()),
                            },
                            Field::Keyed {
                                skip: true,
                                flatten: false,
                                key: "proxy".into(),
                                ty: Type::Named("session.ProxyConfiguration".into()),
                            },
                            Field::Keyed {
                                skip: true,
                                flatten: false,
                                key: "unhandledPromptBehavior".into(),
                                ty: Type::Named("session.UserPromptHandler".into()),
                            },
                            Field::Keyed {
                                skip: true,
                                flatten: false,
                                key: "webSocketUrl".into(),
                                ty: Type::Named("text".into()),
                            },
                            Field::Inline(Type::Named("Extensible".into())),
                        ]),
                    },
                ]),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_range_float() {
        let input = "browsingContext.ImageFormat = {
   type: text,
   ? quality: 0.0..1.0,
}";
        let expected = Ok((
            "",
            Rule {
                name: "browsingContext.ImageFormat".into(),
                ty: Type::Map(vec![
                    Field::Keyed {
                        skip: false,
                        flatten: false,
                        key: "type".into(),
                        ty: Type::Named("text".into()),
                    },
                    Field::Keyed {
                        skip: true,
                        flatten: false,
                        key: "quality".into(),
                        ty: Type::Named(Name::Primitive(Primitive::Float)),
                    },
                ]),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_range_int() {
        let input = "js-int = -9007199254740991..9007199254740991";
        let expected = Ok((
            "",
            Rule {
                name: "js-int".into(),
                ty: Type::Named(Name::Primitive(Primitive::Int)),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_range_uint() {
        let input = "js-uint = 0..9007199254740991";
        let expected = Ok((
            "",
            Rule {
                name: "js-uint".into(),
                ty: Type::Named(Name::Primitive(Primitive::Uint)),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_annotation() {
        let input = "browsingContext.PrintMarginParameters = {
  ? bottom: (float .ge 0.0) .default 1.0,
  ? left: (float .ge 0.0) .default 1.0,
  ? right: (float .ge 0.0) .default 1.0,
  ? top: (float .ge 0.0) .default 1.0,
}";
        let expected = Ok((
            "",
            Rule {
                name: "browsingContext.PrintMarginParameters".into(),
                ty: Type::Map(vec![
                    Field::Keyed {
                        skip: true,
                        flatten: false,
                        key: "bottom".into(),
                        ty: Type::Named(Name::Primitive(Primitive::Float)),
                    },
                    Field::Keyed {
                        skip: true,
                        flatten: false,
                        key: "left".into(),
                        ty: Type::Named(Name::Primitive(Primitive::Float)),
                    },
                    Field::Keyed {
                        skip: true,
                        flatten: false,
                        key: "right".into(),
                        ty: Type::Named(Name::Primitive(Primitive::Float)),
                    },
                    Field::Keyed {
                        skip: true,
                        flatten: false,
                        key: "top".into(),
                        ty: Type::Named(Name::Primitive(Primitive::Float)),
                    },
                ]),
            },
        ));
        let actual = rule(input);
        assert_eq!(actual, expected);
    }
}
