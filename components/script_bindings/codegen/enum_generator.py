# =====================================================================
# 파일 경로: components/script_bindings/codegen/enum_generator.py
# =====================================================================
from __future__ import annotations
from WebIDL import IDLEnum
from configuration import Configuration

# 기존 codegen에 있는 트리 빌더 컴포넌트들을 가져옵니다.
from codegen import CGThing, CGList, CGGeneric, CGNamespace, CGIndenter, getEnumValueName


class CGEnum(CGThing):
    def __init__(self, enum: IDLEnum, config: Configuration) -> None:
        CGThing.__init__(self)

        ident = enum.identifier.name
        enums = ",\n    ".join(map(getEnumValueName, list(enum.values())))
        derives = ["Copy", "Clone", "Debug", "JSTraceable", "MallocSizeOf", "PartialEq"]
        enum_config = config.getEnumConfig(ident)
        extra_derives = enum_config.get("derives", [])
        derives = ", ".join(derives + extra_derives)
        decl = f"""
#[repr(usize)]
#[derive({derives})]
pub enum {ident} {{
    {enums}
}}
"""

        pairs = ",\n    ".join([f'("{val}", super::{ident}::{getEnumValueName(val)})' for val in list(enum.values())])

        inner = f"""
use crate::utils::find_enum_value;
use crate::cformat;
use js::conversions::ConversionResult;
use js::conversions::FromJSValConvertible;
use js::conversions::ToJSValConvertible;
use js::context::RawJSContext;
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
    unsafe fn to_jsval(&self, cx: *mut RawJSContext, rval: MutableHandleValue) {{
        pairs[*self as usize].0.to_jsval(cx, rval);
    }}
}}

impl FromJSValConvertible for super::{ident} {{
    type Config = ();
    unsafe fn from_jsval(cx: *mut RawJSContext, value: HandleValue, _option: ())
                         -> Result<ConversionResult<super::{ident}>, ()> {{
        match find_enum_value(cx, value, pairs) {{
            Err(_) => Err(()),
            Ok((None, search)) => {{
                Ok(ConversionResult::Failure(
                    cformat!("'{{}}' is not a valid enum value for enumeration '{ident}'.", search).into()
                ))
            }}
            Ok((Some(&value), _)) => Ok(ConversionResult::Success(value)),
        }}
    }}
}}
    """
        self.cgRoot = CGList(
            [
                CGGeneric(decl),
                CGNamespace.build([f"{ident}Values"], CGIndenter(CGGeneric(inner)), public=True),
            ]
        )

    def define(self) -> str:
        return self.cgRoot.define()
