import re

from components.script_bindings.codegen.base import CGThing, CGList, CGGeneric, CGWrapper
from components.script_bindings.codegen.codegen import CGNamespace, CGIndenter

from components.script_bindings.codegen.configuration import MakeNativeName


def getEnumValueName(value):
    # Some enum values can be empty strings.  Others might have weird
    # characters in them.  Deal with the former by returning "_empty",
    # deal with possible name collisions from that by throwing if the
    # enum value is actually "_empty", and throw on any value
    # containing non-ASCII chars for now. Replace all chars other than
    # [0-9A-Za-z_] with '_'.
    if re.match("[^\x20-\x7E]", value):
        raise SyntaxError(f'Enum value "{value}" contains non-ASCII characters')
    if re.match("^[0-9]", value):
        value = '_' + value
    value = re.sub(r'[^0-9A-Za-z_]', '_', value)
    if re.match("^_[A-Z]|__", value):
        raise SyntaxError(f'Enum value "{value}" is reserved by the C++ spec')
    if value == "_empty":
        raise SyntaxError('"_empty" is not an IDL enum value we support yet')
    if value == "":
        return "_empty"
    return MakeNativeName(value)


class CGEnum(CGThing):
    def __init__(self, enum, config):
        CGThing.__init__(self)

        ident = enum.identifier.name
        enums = ",\n    ".join(map(getEnumValueName, list(enum.values())))
        derives = ["Copy", "Clone", "Debug", "JSTraceable", "MallocSizeOf", "PartialEq"]
        enum_config = config.getEnumConfig(ident)
        extra_derives = enum_config.get('derives', [])
        derives = ', '.join(derives + extra_derives)
        decl = f"""
#[repr(usize)]
#[derive({derives})]
pub enum {ident} {{
    {enums}
}}
"""

        pairs = ",\n    ".join([f'("{val}", super::{ident}::{getEnumValueName(val)})'
                                for val in list(enum.values())])

        inner = f"""
use crate::utils::find_enum_value;
use js::conversions::ConversionResult;
use js::conversions::FromJSValConvertible;
use js::conversions::ToJSValConvertible;
use js::jsapi::JSContext;
use js::rust::HandleValue;
use js::rust::MutableHandleValue;
use js::jsval::JSVal;

pub(crate) const pairs: &[(&str, super::{ident})] = &[
    {pairs},
];

impl super::{ident} {{
    pub fn as_str(&self) -> &'static str {{
        pairs[*self as usize].0
    }}
}}

impl Default for super::{ident} {{
    fn default() -> super::{ident} {{
        pairs[0].1
    }}
}}

impl std::str::FromStr for super::{ident} {{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {{
        pairs
            .iter()
            .find(|&&(key, _)| s == key)
            .map(|&(_, ev)| ev)
            .ok_or(())
    }}
}}

impl ToJSValConvertible for super::{ident} {{
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {{
        pairs[*self as usize].0.to_jsval(cx, rval);
    }}
}}

impl FromJSValConvertible for super::{ident} {{
    type Config = ();
    unsafe fn from_jsval(cx: *mut JSContext, value: HandleValue, _option: ())
                         -> Result<ConversionResult<super::{ident}>, ()> {{
        match find_enum_value(cx, value, pairs) {{
            Err(_) => Err(()),
            Ok((None, search)) => {{
                Ok(ConversionResult::Failure(
                    format!("'{{}}' is not a valid enum value for enumeration '{ident}'.", search).into()
                ))
            }}
            Ok((Some(&value), _)) => Ok(ConversionResult::Success(value)),
        }}
    }}
}}
    """
        self.cgRoot = CGList([
            CGGeneric(decl),
            CGNamespace.build([f"{ident}Values"],
                              CGIndenter(CGGeneric(inner)), public=True),
        ])

    def define(self):
        return self.cgRoot.define()


class CGNonNamespacedEnum(CGThing):
    def __init__(self, enumName, names, first, comment="", deriving="", repr=""):
        # Account for first value
        entries = [f"{names[0]} = {first}"] + names[1:]

        # Append a Last.
        entries.append(f'#[allow(dead_code)] Last = {first + len(entries)}')

        # Indent.
        entries = [f'    {e}' for e in entries]

        # Build the enum body.
        joinedEntries = ',\n'.join(entries)
        enumstr = f"{comment}pub enum {enumName} {{\n{joinedEntries}\n}}\n"
        if repr:
            enumstr = f"#[repr({repr})]\n{enumstr}"
        if deriving:
            enumstr = f"#[derive({deriving})]\n{enumstr}"
        curr = CGGeneric(enumstr)

        # Add some whitespace padding.
        curr = CGWrapper(curr, pre='\n', post='\n')

        # Add the typedef
        # typedef = '\ntypedef %s::%s %s;\n\n' % (namespace, enumName, enumName)
        # curr = CGList([curr, CGGeneric(typedef)])

        # Save the result.
        self.node = curr

    def define(self):
        return self.node.define()
