use serde::{Deserialize, Serialize};

pub mod bidi {
    use crate::bidi::script::{
        BigIntValue, BooleanValue, NullValue, NumberValue, NumberValueValue, ObjectRemoteValue,
        PrimitiveProtocolValue,
        SpecialNumber::{Infinity, NaN, NegInfinity, NegZero},
        StringValue, UndefinedValue,
    };

    include!(concat!(env!("OUT_DIR"), "/webdriver_bidi.rs"));

    impl Default for script::SerializationOptions {
        fn default() -> Self {
            Self {
                max_dom_depth: Some(0),
                max_object_depth: None,
                include_shadow_tree: Some(script::SerializationOptionsIncludeShadowTree::None),
            }
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#serialize-as-a-remote-value>
    pub fn serialize_as_a_remote_value(
        value: &impl SerializeRemoteValue,
        serialization_options: &script::SerializationOptions,
        ownership_type: script::ResultOwnership,
        // serialization_internal_map: (),
        // realm: String,
        // session: String,
    ) -> script::RemoteValue {
        // 1.
        let remote_value = serialize_primitive_protocol_value(value);
        // 2.
        if let Some(remote_value) = remote_value {
            return script::RemoteValue::PrimitiveProtocolValue(remote_value);
        }
        // 3. TODO
        let handle = None;
        // 4. TODO
        // 5. TODO
        // 6. TODO
        let remote_value = match value {
            // 6.Symbol.
            v if v.is_symbol() => {
                script::RemoteValue::SymbolRemoteValue(script::SymbolRemoteValue {
                    handle,
                    internal_id: None,
                })
            },
            // 6.Array. TODO
            // 6.RegExp.
            v if let Some((pattern, flags)) = v.to_reg_exp() => {
                script::RemoteValue::RegExpRemoteValue(script::RegExpRemoteValue {
                    reg_exp_local_value: script::RegExpLocalValue {
                        value: script::RegExpValue { pattern, flags },
                    },
                    handle,
                    internal_id: None,
                })
            },
            // 6.Date.
            v if let Some(value) = v.to_date() => {
                script::RemoteValue::DateRemoteValue(script::DateRemoteValue {
                    date_local_value: script::DateLocalValue { value },
                    handle,
                    internal_id: None,
                })
            },
            // 6.Map. TODO
            // 6.Set. TODO
            // 6.WeakMap. TODO
            // 6.WeakSet. TODO
            // 6.Generator.
            v if v.is_generator() => {
                script::RemoteValue::GeneratorRemoteValue(script::GeneratorRemoteValue {
                    handle,
                    internal_id: None,
                })
            },
            // 6.Error.
            v if v.is_error() => script::RemoteValue::ErrorRemoteValue(script::ErrorRemoteValue {
                handle,
                internal_id: None,
            }),
            // 6.Proxy.
            v if v.is_proxy() => script::RemoteValue::ProxyRemoteValue(script::ProxyRemoteValue {
                handle,
                internal_id: None,
            }),
            // 6.Promise.
            v if v.is_promise() => {
                script::RemoteValue::PromiseRemoteValue(script::PromiseRemoteValue {
                    handle,
                    internal_id: None,
                })
            },
            // 6.TypedArray.
            v if v.is_type_array() => {
                script::RemoteValue::TypedArrayRemoteValue(script::TypedArrayRemoteValue {
                    handle,
                    internal_id: None,
                })
            },
            // 6.ArrayBuffer.
            v if v.is_array_buffer() => {
                script::RemoteValue::ArrayBufferRemoteValue(script::ArrayBufferRemoteValue {
                    handle,
                    internal_id: None,
                })
            },
            // 6.NodeList. TODO
            // 6.HTMLCollection. TODO
            // 6.Node. TODO
            // 6.WindowProxy. TODO
            // 6.platformobject TODO
            // 6.Callable.
            v if v.is_callable() => {
                script::RemoteValue::FunctionRemoteValue(script::FunctionRemoteValue {
                    handle,
                    internal_id: None,
                })
            },
            // 6.Otherwise.
            _ => {
                // 6.Otherwise.1. Skip Assert.
                // 6.Otherwise.2.
                let mut remote_value = ObjectRemoteValue {
                    handle,
                    internal_id: None,
                    value: None,
                };
                // 6.Otherwise.3. TODO
                // 6.Otherwise.4.
                let serialized = None;
                // 6.Otherwise.5. TODO
                // 6.Otherwise.6. TODO
                if let Some(serialized) = serialized {
                    remote_value.value = Some(serialized);
                }
                script::RemoteValue::ObjectRemoteValue(remote_value)
            },
        };
        // 7.
        remote_value
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#serialize-primitive-protocol-value>
    pub fn serialize_primitive_protocol_value(
        value: &impl SerializeRemoteValue,
    ) -> Option<PrimitiveProtocolValue> {
        // 1.
        let mut remote_value = None;
        // 2.
        // 2.undefined.
        if value.is_undefined() {
            remote_value = Some(UndefinedValue {}.into());
        }
        // 2.null.
        if value.is_null() {
            remote_value = Some(NullValue {}.into())
        }
        // 2.String.
        if let Some(value) = value.to_string() {
            remote_value = Some(StringValue { value }.into())
        }
        // 2.Number.
        if let Some(value) = value.to_number() {
            // 2.Number.1.
            let serialized = match value {
                v if v.is_nan() => NumberValueValue::SpecialNumber(NaN),
                v if v == 0.0 && v.is_sign_negative() => NumberValueValue::SpecialNumber(NegZero),
                f64::INFINITY => NumberValueValue::SpecialNumber(Infinity),
                f64::NEG_INFINITY => NumberValueValue::SpecialNumber(NegInfinity),
                _ => NumberValueValue::Number(value),
            };
            // 2.Number.2.
            remote_value = Some(NumberValue { value: serialized }.into());
        }
        // 2.Boolean.
        if let Some(value) = value.to_boolean() {
            remote_value = Some(BooleanValue { value }.into());
        }
        // 2.BigInt.
        if let Some(value) = value.to_big_int() {
            remote_value = Some(BigIntValue { value }.into());
        }
        // 3.
        remote_value
    }

    /// A value that can be serialized by [`serialize_as_a_remote_value`]
    pub trait SerializeRemoteValue {
        fn is_undefined(&self) -> bool;
        fn is_null(&self) -> bool;
        fn to_string(&self) -> Option<String>;
        fn to_number(&self) -> Option<f64>;
        fn to_boolean(&self) -> Option<bool>;
        fn to_big_int(&self) -> Option<String>;
        fn is_symbol(&self) -> bool;
        fn is_array(&self) -> bool;
        fn to_reg_exp(&self) -> Option<(String, Option<String>)>;
        fn to_date(&self) -> Option<String>;
        fn is_map(&self) -> bool;
        fn is_set(&self) -> bool;
        fn is_weak_map(&self) -> bool;
        fn is_weak_set(&self) -> bool;
        fn is_generator(&self) -> bool;
        fn is_error(&self) -> bool;
        fn is_proxy(&self) -> bool;
        fn is_promise(&self) -> bool;
        fn is_type_array(&self) -> bool;
        fn is_array_buffer(&self) -> bool;
        fn is_node_list(&self) -> bool;
        fn is_html_collection(&self) -> bool;
        fn is_node(&self) -> bool;
        fn is_window_proxy(&self) -> bool;
        fn is_platform_object(&self) -> bool;
        fn is_callable(&self) -> bool;
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#get-the-source>
    pub fn get_the_source() -> script::Source {
        todo!()
    }

    // TODO: these impl From should be derived rather than manually written here.
    impl From<UndefinedValue> for PrimitiveProtocolValue {
        fn from(value: UndefinedValue) -> Self {
            Self::UndefinedValue(value)
        }
    }

    impl From<NullValue> for PrimitiveProtocolValue {
        fn from(value: NullValue) -> Self {
            Self::NullValue(value)
        }
    }

    impl From<StringValue> for PrimitiveProtocolValue {
        fn from(value: StringValue) -> Self {
            Self::StringValue(value)
        }
    }

    impl From<NumberValue> for PrimitiveProtocolValue {
        fn from(value: NumberValue) -> Self {
            Self::NumberValue(value)
        }
    }

    impl From<BooleanValue> for PrimitiveProtocolValue {
        fn from(value: BooleanValue) -> Self {
            Self::BooleanValue(value)
        }
    }

    impl From<BigIntValue> for PrimitiveProtocolValue {
        fn from(value: BigIntValue) -> Self {
            Self::BigIntValue(value)
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ScriptToWebDriverMessage {
    ConsoleEntryAdded(bidi::log::ConsoleLogEntry),
}
