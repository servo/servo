from components.script_bindings.codegen.base import CGThing, CGAbstractMethod, CGGeneric, CGAbstractExternMethod
from components.script_bindings.codegen.codegen import DOMClass, FINALIZE_HOOK_NAME, str_to_cstr_ptr, TRACE_HOOK_NAME, \
    str_to_cstr, CONSTRUCT_HOOK_NAME, Argument, CGIndenter, CGProxyIndexedGetter, CGProxyNamedGetter, \
    CGProxyIndexedSetter, CGProxyNamedSetter, CGProxyNamedDeleter
from components.script_bindings.codegen.utils import dedent, stripTrailingWhitespace


class CGDOMJSClass(CGThing):
    """
    Generate a DOMJSClass for a given descriptor
    """
    def __init__(self, descriptor):
        CGThing.__init__(self)
        self.descriptor = descriptor

    def define(self):
        parentName = self.descriptor.getParentName()
        if not parentName:
            parentName = "Reflector"

        args = {
            "domClass": DOMClass(self.descriptor),
            "enumerateHook": "None",
            "finalizeHook": f"{FINALIZE_HOOK_NAME}::<D>",
            "flags": "JSCLASS_FOREGROUND_FINALIZE",
            "name": str_to_cstr_ptr(self.descriptor.interface.identifier.name),
            "resolveHook": "None",
            "mayResolveHook": "None",
            "slots": "1",
            "traceHook": f"{TRACE_HOOK_NAME}::<D>",
        }
        if self.descriptor.isGlobal():
            assert not self.descriptor.weakReferenceable
            args["flags"] = "JSCLASS_IS_GLOBAL | JSCLASS_DOM_GLOBAL | JSCLASS_FOREGROUND_FINALIZE"
            args["slots"] = "JSCLASS_GLOBAL_SLOT_COUNT + 1"
            if self.descriptor.interface.getExtendedAttribute("NeedResolve"):
                args["enumerateHook"] = "Some(enumerate_window::<D>)"
                args["resolveHook"] = "Some(resolve_window::<D>)"
                args["mayResolveHook"] = "Some(may_resolve_window::<D>)"
            else:
                args["enumerateHook"] = "Some(enumerate_global)"
                args["resolveHook"] = "Some(resolve_global)"
                args["mayResolveHook"] = "Some(may_resolve_global)"
            args["traceHook"] = "js::jsapi::JS_GlobalObjectTraceHook"
        elif self.descriptor.weakReferenceable:
            args["slots"] = "2"
        return f"""
static CLASS_OPS: ThreadUnsafeOnceLock<JSClassOps> = ThreadUnsafeOnceLock::new();

pub(crate) fn init_class_ops<D: DomTypes>() {{
    CLASS_OPS.set(JSClassOps {{
        addProperty: None,
        delProperty: None,
        enumerate: None,
        newEnumerate: {args['enumerateHook']},
        resolve: {args['resolveHook']},
        mayResolve: {args['mayResolveHook']},
        finalize: Some({args['finalizeHook']}),
        call: None,
        construct: None,
        trace: Some({args['traceHook']}),
    }});
}}

pub static Class: ThreadUnsafeOnceLock<DOMJSClass> = ThreadUnsafeOnceLock::new();

pub(crate) fn init_domjs_class<D: DomTypes>() {{
    init_class_ops::<D>();
    Class.set(DOMJSClass {{
        base: JSClass {{
            name: {args['name']},
            flags: JSCLASS_IS_DOMJSCLASS | {args['flags']} |
                   ((({args['slots']}) & JSCLASS_RESERVED_SLOTS_MASK) << JSCLASS_RESERVED_SLOTS_SHIFT)
                   /* JSCLASS_HAS_RESERVED_SLOTS({args['slots']}) */,
            cOps: unsafe {{ CLASS_OPS.get() }},
            spec: ptr::null(),
            ext: ptr::null(),
            oOps: ptr::null(),
        }},
        dom_class: {args['domClass']},
    }});
}}
"""


class CGPrototypeJSClass(CGThing):
    def __init__(self, descriptor):
        CGThing.__init__(self)
        self.descriptor = descriptor

    def define(self):
        name = str_to_cstr_ptr(f"{self.descriptor.interface.identifier.name}Prototype")
        slotCount = 0
        if self.descriptor.hasLegacyUnforgeableMembers:
            slotCount += 1
        slotCountStr = f"{slotCount} & JSCLASS_RESERVED_SLOTS_MASK" if slotCount > 0 else "0"
        return f"""
static PrototypeClass: JSClass = JSClass {{
    name: {name},
    flags:
        // JSCLASS_HAS_RESERVED_SLOTS()
        ({slotCountStr} ) << JSCLASS_RESERVED_SLOTS_SHIFT,
    cOps: ptr::null(),
    spec: ptr::null(),
    ext: ptr::null(),
    oOps: ptr::null(),
}};
"""


class CGInterfaceObjectJSClass(CGThing):
    def __init__(self, descriptor):
        assert descriptor.interface.hasInterfaceObject() and not descriptor.interface.isCallback()
        CGThing.__init__(self)
        self.descriptor = descriptor

    def define(self):
        if self.descriptor.interface.isNamespace():
            classString = self.descriptor.interface.getExtendedAttribute("ClassString")
            if classString:
                classString = classString[0]
            else:
                classString = "Object"
            return f"""
static NAMESPACE_OBJECT_CLASS: NamespaceObjectClass = unsafe {{
    NamespaceObjectClass::new({str_to_cstr(classString)})
}};
"""
        if self.descriptor.interface.ctor():
            constructorBehavior = f"InterfaceConstructorBehavior::call({CONSTRUCT_HOOK_NAME}::<D>)"
        else:
            constructorBehavior = "InterfaceConstructorBehavior::throw()"
        name = self.descriptor.interface.identifier.name
        representation = f'b"function {name}() {{\\n    [native code]\\n}}"'
        return f"""
static INTERFACE_OBJECT_CLASS: ThreadUnsafeOnceLock<NonCallbackInterfaceObjectClass> = ThreadUnsafeOnceLock::new();

pub(crate) fn init_interface_object<D: DomTypes>() {{
    INTERFACE_OBJECT_CLASS.set(NonCallbackInterfaceObjectClass::new(
        Box::leak(Box::new({constructorBehavior})),
        {representation},
        PrototypeList::ID::{name},
        {self.descriptor.prototypeDepth},
    ));
}}
"""


class CGDefineProxyHandler(CGAbstractMethod):
    """
    A method to create and cache the proxy trap for a given interface.
    """
    def __init__(self, descriptor):
        assert descriptor.proxy
        CGAbstractMethod.__init__(self, descriptor, 'DefineProxyHandler',
                                  '*const libc::c_void', [],
                                  pub=True, unsafe=True, templateArgs=["D: DomTypes"])

    def define(self):
        return CGAbstractMethod.define(self)

    def definition_body(self):
        customDefineProperty = 'proxyhandler::define_property'
        if self.descriptor.isMaybeCrossOriginObject() or self.descriptor.operations['IndexedSetter'] or \
           self.descriptor.operations['NamedSetter']:
            customDefineProperty = 'defineProperty::<D>'

        customDelete = 'proxyhandler::delete'
        if self.descriptor.isMaybeCrossOriginObject() or self.descriptor.operations['NamedDeleter']:
            customDelete = 'delete::<D>'

        customGetPrototypeIfOrdinary = 'Some(proxyhandler::get_prototype_if_ordinary)'
        customGetPrototype = 'None'
        customSetPrototype = 'None'
        if self.descriptor.isMaybeCrossOriginObject():
            customGetPrototypeIfOrdinary = 'Some(proxyhandler::maybe_cross_origin_get_prototype_if_ordinary_rawcx)'
            customGetPrototype = 'Some(getPrototype::<D>)'
            customSetPrototype = 'Some(proxyhandler::maybe_cross_origin_set_prototype_rawcx)'
        # The base class `BaseProxyHandler`'s `setImmutablePrototype` (not to be
        # confused with ECMAScript's `[[SetImmutablePrototype]]`) always fails.
        # This is the desired behavior, so we don't override it.

        customSet = 'None'
        if self.descriptor.isMaybeCrossOriginObject():
            # `maybe_cross_origin_set_rawcx` doesn't support legacy platform objects'
            # `[[Set]]` (https://heycam.github.io/webidl/#legacy-platform-object-set) (yet).
            assert not self.descriptor.operations['IndexedGetter']
            assert not self.descriptor.operations['NamedGetter']
            customSet = 'Some(proxyhandler::maybe_cross_origin_set_rawcx::<D>)'

        getOwnEnumerablePropertyKeys = "own_property_keys::<D>"
        if self.descriptor.interface.getExtendedAttribute("LegacyUnenumerableNamedProperties") or \
           self.descriptor.isMaybeCrossOriginObject():
            getOwnEnumerablePropertyKeys = "getOwnEnumerablePropertyKeys::<D>"

        return CGGeneric(f"""
init_proxy_handler_dom_class::<D>();

let traps = ProxyTraps {{
    enter: None,
    getOwnPropertyDescriptor: Some(getOwnPropertyDescriptor::<D>),
    defineProperty: Some({customDefineProperty}),
    ownPropertyKeys: Some(own_property_keys::<D>),
    delete_: Some({customDelete}),
    enumerate: None,
    getPrototypeIfOrdinary: {customGetPrototypeIfOrdinary},
    getPrototype: {customGetPrototype},
    setPrototype: {customSetPrototype},
    setImmutablePrototype: None,
    preventExtensions: Some(proxyhandler::prevent_extensions),
    isExtensible: Some(proxyhandler::is_extensible),
    has: None,
    get: Some(get::<D>),
    set: {customSet},
    call: None,
    construct: None,
    hasOwn: Some(hasOwn::<D>),
    getOwnEnumerablePropertyKeys: Some({getOwnEnumerablePropertyKeys}),
    nativeCall: None,
    objectClassIs: None,
    className: Some(className),
    fun_toString: None,
    boxedValue_unbox: None,
    defaultValue: None,
    trace: Some({TRACE_HOOK_NAME}::<D>),
    finalize: Some({FINALIZE_HOOK_NAME}::<D>),
    objectMoved: None,
    isCallable: None,
    isConstructor: None,
}};

CreateProxyHandler(&traps, unsafe {{ Class.get() }}.as_void_ptr())\
""")


class CGDOMJSProxyHandler_getOwnPropertyDescriptor(CGAbstractExternMethod):
    def __init__(self, descriptor):
        args = [Argument('*mut JSContext', 'cx'), Argument('RawHandleObject', 'proxy'),
                Argument('RawHandleId', 'id'),
                Argument('RawMutableHandle<PropertyDescriptor>', 'mut desc'),
                Argument('*mut bool', 'is_none')]
        CGAbstractExternMethod.__init__(self, descriptor, "getOwnPropertyDescriptor",
                                        "bool", args, templateArgs=['D: DomTypes'])
        self.descriptor = descriptor

    # https://heycam.github.io/webidl/#LegacyPlatformObjectGetOwnProperty
    def getBody(self):
        indexedGetter = self.descriptor.operations['IndexedGetter']

        get = "let cx = SafeJSContext::from_ptr(cx);\n"

        if self.descriptor.isMaybeCrossOriginObject():
            get += dedent(
                """
                if !<D as DomHelpers<D>>::is_platform_object_same_origin(cx, proxy) {
                    if !proxyhandler::cross_origin_get_own_property_helper(
                        cx, proxy, CROSS_ORIGIN_PROPERTIES.get(), id, desc, &mut *is_none
                    ) {
                        return false;
                    }
                    if *is_none {
                        return proxyhandler::cross_origin_property_fallback::<D>(cx, proxy, id, desc, &mut *is_none);
                    }
                    return true;
                }

                // Safe to enter the Realm of proxy now.
                let _ac = JSAutoRealm::new(*cx, proxy.get());
                """)

        if indexedGetter:
            get += "let index = get_array_index_from_id(Handle::from_raw(id));\n"

            attrs = "JSPROP_ENUMERATE"
            if self.descriptor.operations['IndexedSetter'] is None:
                attrs += " | JSPROP_READONLY"
            fillDescriptor = ("set_property_descriptor(\n"
                              "    MutableHandle::from_raw(desc),\n"
                              "    rval.handle(),\n"
                              f"    ({attrs}) as u32,\n"
                              "    &mut *is_none\n"
                              ");\n"
                              "return true;")
            templateValues = {
                'jsvalRef': 'rval.handle_mut()',
                'successCode': fillDescriptor,
                'pre': 'rooted!(in(*cx) let mut rval = UndefinedValue());'
            }
            get += ("if let Some(index) = index {\n"
                    "    let this = UnwrapProxy::<D>(proxy);\n"
                    "    let this = &*this;\n"
                    f"{CGIndenter(CGProxyIndexedGetter(self.descriptor, templateValues)).define()}\n"
                    "}\n")

        if self.descriptor.supportsNamedProperties():
            attrs = []
            if not self.descriptor.interface.getExtendedAttribute("LegacyUnenumerableNamedProperties"):
                attrs.append("JSPROP_ENUMERATE")
            if self.descriptor.operations['NamedSetter'] is None:
                attrs.append("JSPROP_READONLY")
            if attrs:
                attrs = " | ".join(attrs)
            else:
                attrs = "0"
            fillDescriptor = ("set_property_descriptor(\n"
                              "    MutableHandle::from_raw(desc),\n"
                              "    rval.handle(),\n"
                              f"    ({attrs}) as u32,\n"
                              "    &mut *is_none\n"
                              ");\n"
                              "return true;")
            templateValues = {
                'jsvalRef': 'rval.handle_mut()',
                'successCode': fillDescriptor,
                'pre': 'rooted!(in(*cx) let mut rval = UndefinedValue());'
            }

            # See the similar-looking in CGDOMJSProxyHandler_get for the spec quote.
            condition = "id.is_string() || id.is_int()"
            if indexedGetter:
                condition = f"index.is_none() && ({condition})"
            # Once we start supporting OverrideBuiltins we need to make
            # ResolveOwnProperty or EnumerateOwnProperties filter out named
            # properties that shadow prototype properties.
            namedGet = f"""
if {condition} {{
    let mut has_on_proto = false;
    if !has_property_on_prototype(*cx, proxy_lt, id_lt, &mut has_on_proto) {{
        return false;
    }}
    if !has_on_proto {{
        {CGIndenter(CGProxyNamedGetter(self.descriptor, templateValues), 8).define()}
    }}
}}
"""
        else:
            namedGet = ""

        return f"""{get}\
rooted!(in(*cx) let mut expando = ptr::null_mut::<JSObject>());
get_expando_object(proxy, expando.handle_mut());
//if (!xpc::WrapperFactory::IsXrayWrapper(proxy) && (expando = GetExpandoObject(proxy))) {{
let proxy_lt = Handle::from_raw(proxy);
let id_lt = Handle::from_raw(id);
if !expando.is_null() {{
    rooted!(in(*cx) let mut ignored = ptr::null_mut::<JSObject>());
    if !JS_GetPropertyDescriptorById(*cx, expando.handle().into(), id, desc, ignored.handle_mut().into(), is_none) {{
        return false;
    }}
    if !*is_none {{
        // Pretend the property lives on the wrapper.
        return true;
    }}
}}
{namedGet}\
true"""

    def definition_body(self):
        return CGGeneric(self.getBody())


class CGDOMJSProxyHandler_defineProperty(CGAbstractExternMethod):
    def __init__(self, descriptor):
        args = [Argument('*mut JSContext', 'cx'), Argument('RawHandleObject', 'proxy'),
                Argument('RawHandleId', 'id'),
                Argument('RawHandle<PropertyDescriptor>', 'desc'),
                Argument('*mut ObjectOpResult', 'opresult')]
        CGAbstractExternMethod.__init__(self, descriptor, "defineProperty", "bool", args, templateArgs=['D: DomTypes'])
        self.descriptor = descriptor

    def getBody(self):
        set = "let cx = SafeJSContext::from_ptr(cx);\n"

        if self.descriptor.isMaybeCrossOriginObject():
            set += dedent(
                """
                if !<D as DomHelpers<D>>::is_platform_object_same_origin(cx, proxy) {
                    return proxyhandler::report_cross_origin_denial::<D>(cx, id, "define");
                }

                // Safe to enter the Realm of proxy now.
                let _ac = JSAutoRealm::new(*cx, proxy.get());
                """)

        indexedSetter = self.descriptor.operations['IndexedSetter']
        if indexedSetter:
            set += ("let index = get_array_index_from_id(Handle::from_raw(id));\n"
                    "if let Some(index) = index {\n"
                    "    let this = UnwrapProxy::<D>(proxy);\n"
                    "    let this = &*this;\n"
                    f"{CGIndenter(CGProxyIndexedSetter(self.descriptor)).define()}"
                    "    return (*opresult).succeed();\n"
                    "}\n")
        elif self.descriptor.operations['IndexedGetter']:
            set += ("if get_array_index_from_id(Handle::from_raw(id)).is_some() {\n"
                    "    return (*opresult).failNoIndexedSetter();\n"
                    "}\n")

        namedSetter = self.descriptor.operations['NamedSetter']
        if namedSetter:
            if self.descriptor.hasLegacyUnforgeableMembers:
                raise TypeError("Can't handle a named setter on an interface that has "
                                "unforgeables. Figure out how that should work!")
            set += ("if id.is_string() || id.is_int() {\n"
                    f"{CGIndenter(CGProxyNamedSetter(self.descriptor)).define()}"
                    "    return (*opresult).succeed();\n"
                    "}\n")
        elif self.descriptor.supportsNamedProperties():
            set += ("if id.is_string() || id.is_int() {\n"
                    f"{CGIndenter(CGProxyNamedGetter(self.descriptor)).define()}"
                    "    if result.is_some() {\n"
                    "        return (*opresult).fail_no_named_setter();\n"
                    "    }\n"
                    "}\n")
        set += f"return proxyhandler::define_property(*cx, {', '.join(a.name for a in self.args[1:])});"
        return set

    def definition_body(self):
        return CGGeneric(self.getBody())


class CGDOMJSProxyHandler_delete(CGAbstractExternMethod):
    def __init__(self, descriptor):
        args = [Argument('*mut JSContext', 'cx'), Argument('RawHandleObject', 'proxy'),
                Argument('RawHandleId', 'id'),
                Argument('*mut ObjectOpResult', 'res')]
        CGAbstractExternMethod.__init__(self, descriptor, "delete", "bool", args, templateArgs=['D: DomTypes'])
        self.descriptor = descriptor

    def getBody(self):
        set = "let cx = SafeJSContext::from_ptr(cx);\n"

        if self.descriptor.isMaybeCrossOriginObject():
            set += dedent(
                """
                if !<D as DomHelpers<D>>::is_platform_object_same_origin(cx, proxy) {
                    return proxyhandler::report_cross_origin_denial::<D>(cx, id, "delete");
                }

                // Safe to enter the Realm of proxy now.
                let _ac = JSAutoRealm::new(*cx, proxy.get());
                """)

        if self.descriptor.operations['NamedDeleter']:
            if self.descriptor.hasLegacyUnforgeableMembers:
                raise TypeError("Can't handle a deleter on an interface that has "
                                "unforgeables. Figure out how that should work!")
            set += CGProxyNamedDeleter(self.descriptor).define()
        set += f"return proxyhandler::delete(*cx, {', '.join(a.name for a in self.args[1:])});"
        return set

    def definition_body(self):
        return CGGeneric(self.getBody())


class CGDOMJSProxyHandler_ownPropertyKeys(CGAbstractExternMethod):
    def __init__(self, descriptor):
        args = [Argument('*mut JSContext', 'cx'),
                Argument('RawHandleObject', 'proxy'),
                Argument('RawMutableHandleIdVector', 'props')]
        CGAbstractExternMethod.__init__(self, descriptor, "own_property_keys", "bool", args,
                                        templateArgs=['D: DomTypes'])
        self.descriptor = descriptor

    def getBody(self):
        body = dedent(
            """
            let cx = SafeJSContext::from_ptr(cx);
            let unwrapped_proxy = UnwrapProxy::<D>(proxy);
            """)

        if self.descriptor.isMaybeCrossOriginObject():
            body += dedent(
                """
                if !<D as DomHelpers<D>>::is_platform_object_same_origin(cx, proxy) {
                    return proxyhandler::cross_origin_own_property_keys(
                        cx, proxy, CROSS_ORIGIN_PROPERTIES.get(), props
                    );
                }

                // Safe to enter the Realm of proxy now.
                let _ac = JSAutoRealm::new(*cx, proxy.get());
                """)

        if self.descriptor.operations['IndexedGetter']:
            body += dedent(
                """
                for i in 0..(*unwrapped_proxy).Length() {
                    rooted!(in(*cx) let mut rooted_jsid: jsid);
                    int_to_jsid(i as i32, rooted_jsid.handle_mut());
                    AppendToIdVector(props, rooted_jsid.handle());
                }
                """)

        if self.descriptor.supportsNamedProperties():
            body += dedent(
                """
                for name in (*unwrapped_proxy).SupportedPropertyNames() {
                    let cstring = CString::new(name).unwrap();
                    let jsstring = JS_AtomizeAndPinString(*cx, cstring.as_ptr());
                    rooted!(in(*cx) let rooted = jsstring);
                    rooted!(in(*cx) let mut rooted_jsid: jsid);
                    RUST_INTERNED_STRING_TO_JSID(*cx, rooted.handle().get(), rooted_jsid.handle_mut());
                    AppendToIdVector(props, rooted_jsid.handle());
                }
                """)

        body += dedent(
            """
            rooted!(in(*cx) let mut expando = ptr::null_mut::<JSObject>());
            get_expando_object(proxy, expando.handle_mut());
            if !expando.is_null() &&
                !GetPropertyKeys(*cx, expando.handle(), JSITER_OWNONLY | JSITER_HIDDEN | JSITER_SYMBOLS, props) {
                return false;
            }

            true
            """)

        return body

    def definition_body(self):
        return CGGeneric(self.getBody())


class CGDOMJSProxyHandler_getOwnEnumerablePropertyKeys(CGAbstractExternMethod):
    def __init__(self, descriptor):
        assert (descriptor.operations["IndexedGetter"]
                and descriptor.interface.getExtendedAttribute("LegacyUnenumerableNamedProperties")
                or descriptor.isMaybeCrossOriginObject())
        args = [Argument('*mut JSContext', 'cx'),
                Argument('RawHandleObject', 'proxy'),
                Argument('RawMutableHandleIdVector', 'props')]
        CGAbstractExternMethod.__init__(self, descriptor,
                                        "getOwnEnumerablePropertyKeys", "bool", args, templateArgs=['D: DomTypes'])
        self.descriptor = descriptor

    def getBody(self):
        body = dedent(
            """
            let cx = SafeJSContext::from_ptr(cx);
            let unwrapped_proxy = UnwrapProxy::<D>(proxy);
            """)

        if self.descriptor.isMaybeCrossOriginObject():
            body += dedent(
                """
                if !<D as DomHelpers<D>>::is_platform_object_same_origin(cx, proxy) {
                    // There are no enumerable cross-origin props, so we're done.
                    return true;
                }

                // Safe to enter the Realm of proxy now.
                let _ac = JSAutoRealm::new(*cx, proxy.get());
                """)

        if self.descriptor.operations['IndexedGetter']:
            body += dedent(
                """
                for i in 0..(*unwrapped_proxy).Length() {
                    rooted!(in(*cx) let mut rooted_jsid: jsid);
                    int_to_jsid(i as i32, rooted_jsid.handle_mut());
                    AppendToIdVector(props, rooted_jsid.handle());
                }
                """)

        body += dedent(
            """
            rooted!(in(*cx) let mut expando = ptr::null_mut::<JSObject>());
            get_expando_object(proxy, expando.handle_mut());
            if !expando.is_null() &&
                !GetPropertyKeys(*cx, expando.handle(), JSITER_OWNONLY | JSITER_HIDDEN | JSITER_SYMBOLS, props) {
                return false;
            }

            true
            """)

        return body

    def definition_body(self):
        return CGGeneric(self.getBody())


class CGDOMJSProxyHandler_hasOwn(CGAbstractExternMethod):
    def __init__(self, descriptor):
        args = [Argument('*mut JSContext', 'cx'), Argument('RawHandleObject', 'proxy'),
                Argument('RawHandleId', 'id'), Argument('*mut bool', 'bp')]
        CGAbstractExternMethod.__init__(self, descriptor, "hasOwn", "bool", args, templateArgs=['D: DomTypes'])
        self.descriptor = descriptor

    def getBody(self):
        indexedGetter = self.descriptor.operations['IndexedGetter']
        indexed = "let cx = SafeJSContext::from_ptr(cx);\n"

        if self.descriptor.isMaybeCrossOriginObject():
            indexed += dedent(
                """
                if !<D as DomHelpers<D>>::is_platform_object_same_origin(cx, proxy) {
                    return proxyhandler::cross_origin_has_own(
                        cx, proxy, CROSS_ORIGIN_PROPERTIES.get(), id, bp
                    );
                }

                // Safe to enter the Realm of proxy now.
                let _ac = JSAutoRealm::new(*cx, proxy.get());
                """)

        if indexedGetter:
            indexed += ("let index = get_array_index_from_id(Handle::from_raw(id));\n"
                        "if let Some(index) = index {\n"
                        "    let this = UnwrapProxy::<D>(proxy);\n"
                        "    let this = &*this;\n"
                        f"{CGIndenter(CGProxyIndexedGetter(self.descriptor)).define()}\n"
                        "    *bp = result.is_some();\n"
                        "    return true;\n"
                        "}\n\n")

        condition = "id.is_string() || id.is_int()"
        if indexedGetter:
            condition = f"index.is_none() && ({condition})"
        if self.descriptor.supportsNamedProperties():
            named = f"""
if {condition} {{
    let mut has_on_proto = false;
    if !has_property_on_prototype(*cx, proxy_lt, id_lt, &mut has_on_proto) {{
        return false;
    }}
    if !has_on_proto {{
        {CGIndenter(CGProxyNamedGetter(self.descriptor), 8).define()}
        *bp = result.is_some();
        return true;
    }}
}}

"""
        else:
            named = ""

        return f"""{indexed}\
rooted!(in(*cx) let mut expando = ptr::null_mut::<JSObject>());
let proxy_lt = Handle::from_raw(proxy);
let id_lt = Handle::from_raw(id);
get_expando_object(proxy, expando.handle_mut());
if !expando.is_null() {{
    let ok = JS_HasPropertyById(*cx, expando.handle().into(), id, bp);
    if !ok || *bp {{
        return ok;
    }}
}}
{named}\
*bp = false;
true"""

    def definition_body(self):
        return CGGeneric(self.getBody())


class CGDOMJSProxyHandler_get(CGAbstractExternMethod):
    def __init__(self, descriptor):
        args = [Argument('*mut JSContext', 'cx'), Argument('RawHandleObject', 'proxy'),
                Argument('RawHandleValue', 'receiver'), Argument('RawHandleId', 'id'),
                Argument('RawMutableHandleValue', 'vp')]
        CGAbstractExternMethod.__init__(self, descriptor, "get", "bool", args, templateArgs=['D: DomTypes'])
        self.descriptor = descriptor

    # https://heycam.github.io/webidl/#LegacyPlatformObjectGetOwnProperty
    def getBody(self):
        if self.descriptor.isMaybeCrossOriginObject():
            maybeCrossOriginGet = dedent(
                """
                if !<D as DomHelpers<D>>::is_platform_object_same_origin(cx, proxy) {
                    return proxyhandler::cross_origin_get::<D>(cx, proxy, receiver, id, vp);
                }

                // Safe to enter the Realm of proxy now.
                let _ac = JSAutoRealm::new(*cx, proxy.get());
                """)
        else:
            maybeCrossOriginGet = ""
        getFromExpando = """\
rooted!(in(*cx) let mut expando = ptr::null_mut::<JSObject>());
get_expando_object(proxy, expando.handle_mut());
if !expando.is_null() {
    let mut hasProp = false;
    if !JS_HasPropertyById(*cx, expando.handle().into(), id, &mut hasProp) {
        return false;
    }

    if hasProp {
        return JS_ForwardGetPropertyTo(*cx, expando.handle().into(), id, receiver, vp);
    }
}"""

        templateValues = {
            'jsvalRef': 'vp_lt',
            'successCode': 'return true;',
        }

        indexedGetter = self.descriptor.operations['IndexedGetter']
        if indexedGetter:
            getIndexedOrExpando = ("let index = get_array_index_from_id(id_lt);\n"
                                   "if let Some(index) = index {\n"
                                   "    let this = UnwrapProxy::<D>(proxy);\n"
                                   "    let this = &*this;\n"
                                   f"{CGIndenter(CGProxyIndexedGetter(self.descriptor, templateValues)).define()}")
            trimmedGetFromExpando = stripTrailingWhitespace(getFromExpando.replace('\n', '\n    '))
            getIndexedOrExpando += f"""
    // Even if we don't have this index, we don't forward the
    // get on to our expando object.
}} else {{
    {trimmedGetFromExpando}
}}
"""
        else:
            getIndexedOrExpando = f"{getFromExpando}\n"

        if self.descriptor.supportsNamedProperties():
            condition = "id.is_string() || id.is_int()"
            # From step 1:
            #     If O supports indexed properties and P is an array index, then:
            #
            #         3. Set ignoreNamedProps to true.
            if indexedGetter:
                condition = f"index.is_none() && ({condition})"
            getNamed = (f"if {condition} {{\n"
                        f"{CGIndenter(CGProxyNamedGetter(self.descriptor, templateValues)).define()}}}\n")
        else:
            getNamed = ""

        return f"""
//MOZ_ASSERT(!xpc::WrapperFactory::IsXrayWrapper(proxy),
//"Should not have a XrayWrapper here");
let cx = SafeJSContext::from_ptr(cx);

{maybeCrossOriginGet}

let proxy_lt = Handle::from_raw(proxy);
let mut vp_lt = MutableHandle::from_raw(vp);
let id_lt = Handle::from_raw(id);
let receiver_lt = Handle::from_raw(receiver);

{getIndexedOrExpando}
let mut found = false;
if !get_property_on_prototype(*cx, proxy_lt, receiver_lt, id_lt, &mut found, vp_lt.reborrow()) {{
    return false;
}}

if found {{
    return true;
}}
{getNamed}
vp.set(UndefinedValue());
true"""

    def definition_body(self):
        return CGGeneric(self.getBody())


class CGDOMJSProxyHandler_getPrototype(CGAbstractExternMethod):
    def __init__(self, descriptor):
        args = [Argument('*mut JSContext', 'cx'), Argument('RawHandleObject', 'proxy'),
                Argument('RawMutableHandleObject', 'proto')]
        CGAbstractExternMethod.__init__(self, descriptor, "getPrototype", "bool", args, templateArgs=["D: DomTypes"])
        assert descriptor.isMaybeCrossOriginObject()
        self.descriptor = descriptor

    def getBody(self):
        return dedent(
            """
            let cx = SafeJSContext::from_ptr(cx);
            proxyhandler::maybe_cross_origin_get_prototype::<D>(cx, proxy, GetProtoObject::<D>, proto)
            """)

    def definition_body(self):
        return CGGeneric(self.getBody())


class CGDOMJSProxyHandler_className(CGAbstractExternMethod):
    def __init__(self, descriptor):
        args = [Argument('*mut JSContext', 'cx'), Argument('RawHandleObject', '_proxy')]
        CGAbstractExternMethod.__init__(self, descriptor, "className", "*const libc::c_char", args, doesNotPanic=True)
        self.descriptor = descriptor

    def getBody(self):
        return str_to_cstr_ptr(self.descriptor.name)

    def definition_body(self):
        return CGGeneric(self.getBody())
