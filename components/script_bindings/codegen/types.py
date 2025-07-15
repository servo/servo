import string

from components.script_bindings.codegen.base import CGWrapper, CGGeneric
from components.script_bindings.codegen.codegen import JSToNativeConversionInfo, CGIndenter, innerContainerType, \
    wrapInNativeContainerType, getConversionConfigForType, union_native_type, is_typed_array, CGTemplatedType, CGCallbackTempRoot, CGIfElseWrapper, builtinNames, numericTags, \
    type_needs_auto_root, getRetvalDeclarationForType, returnTypeNeedsOutparam
from components.script_bindings.codegen.dictionary import CGDictionary
from components.script_bindings.codegen.enums import getEnumValueName
from components.script_bindings.codegen.configuration import DescriptorProvider
from components.script_bindings.codegen.utils import firstCap, fill, toStringBool


def isDomInterface(t, logging=False):
    while isinstance(t, IDLNullableType) or isinstance(t, IDLWrapperType):
        t = t.inner
    if isinstance(t, IDLInterface):
        return True
    if t.isCallback() or t.isPromise():
        return True
    return t.isInterface() and (t.isGeckoInterface() or (t.isSpiderMonkeyInterface() and not t.isBufferSource()))


def containsDomInterface(t, logging=False):
    if isinstance(t, IDLArgument):
        t = t.type
    if isinstance(t, IDLTypedefType):
        t = t.innerType
    while isinstance(t, IDLNullableType) or isinstance(t, IDLWrapperType):
        t = t.inner
    if t.isEnum():
        return False
    if t.isUnion():
        return any(map(lambda x: containsDomInterface(x), t.flatMemberTypes))
    if t.isDictionary():
        return any(map(lambda x: containsDomInterface(x), t.members)) or (t.parent and containsDomInterface(t.parent))
    if isDomInterface(t):
        return True
    if t.isSequence():
        return containsDomInterface(t.inner)
    return False


def getJSToNativeConversionInfo(type, descriptorProvider, failureCode=None,
                                isDefinitelyObject: bool = False,
                                isMember: bool | str = False,
                                isArgument=False,
                                isAutoRooted=False,
                                invalidEnumValueFatal=True,
                                defaultValue=None,
                                exceptionCode=None,
                                allowTreatNonObjectAsNull: bool = False,
                                isCallbackReturnValue=False,
                                sourceDescription="value") -> JSToNativeConversionInfo:
    """
    Get a template for converting a JS value to a native object based on the
    given type and descriptor.  If failureCode is given, then we're actually
    testing whether we can convert the argument to the desired type.  That
    means that failures to convert due to the JS value being the wrong type of
    value need to use failureCode instead of throwing exceptions.  Failures to
    convert that are due to JS exceptions (from toString or valueOf methods) or
    out of memory conditions need to throw exceptions no matter what
    failureCode is.

    If isDefinitelyObject is True, that means we know the value
    isObject() and we have no need to recheck that.

    isMember is `False`, "Dictionary", "Union" or "Variadic", and affects
    whether this function returns code suitable for an on-stack rooted binding
    or suitable for storing in an appropriate larger structure.

    invalidEnumValueFatal controls whether an invalid enum value conversion
    attempt will throw (if true) or simply return without doing anything (if
    false).

    If defaultValue is not None, it's the IDL default value for this conversion

    If allowTreatNonObjectAsNull is true, then [TreatNonObjectAsNull]
    extended attributes on nullable callback functions will be honored.

    The return value from this function is an object of JSToNativeConversionInfo consisting of four things:

    1)  A string representing the conversion code.  This will have template
        substitution performed on it as follows:

          ${val} replaced by an expression for the JS::Value in question

    2)  A string or None representing Rust code for the default value (if any).

    3)  A CGThing representing the native C++ type we're converting to
        (declType).  This is allowed to be None if the conversion code is
        supposed to be used as-is.

    4)  A boolean indicating whether the caller has to root the result.

    """
    # We should not have a defaultValue if we know we're an object
    assert not isDefinitelyObject or defaultValue is None

    isEnforceRange = type.hasEnforceRange()
    isClamp = type.hasClamp()
    if type.legacyNullToEmptyString:
        treatNullAs = "EmptyString"
    else:
        treatNullAs = "Default"

    # If exceptionCode is not set, we'll just rethrow the exception we got.
    # Note that we can't just set failureCode to exceptionCode, because setting
    # failureCode will prevent pending exceptions from being set in cases when
    # they really should be!
    if exceptionCode is None:
        exceptionCode = "return false;\n"

    if failureCode is None:
        failOrPropagate = f"throw_type_error(*cx, &error);\n{exceptionCode}"
    else:
        failOrPropagate = failureCode

    def handleOptional(template, declType, default):
        assert (defaultValue is None) == (default is None)
        return JSToNativeConversionInfo(template, default, declType)

    # Helper functions for dealing with failures due to the JS value being the
    # wrong type of value.
    def onFailureNotAnObject(failureCode):
        return CGWrapper(
            CGGeneric(
                failureCode
                or (f'throw_type_error(*cx, "{firstCap(sourceDescription)} is not an object.");\n'
                    f'{exceptionCode}')),
            post="\n")

    def onFailureNotCallable(failureCode):
        return CGGeneric(
            failureCode
            or (f'throw_type_error(*cx, \"{firstCap(sourceDescription)} is not callable.\");\n'
                f'{exceptionCode}'))

    # A helper function for handling default values.
    def handleDefault(nullValue):
        if defaultValue is None:
            return None

        if isinstance(defaultValue, IDLNullValue):
            assert type.nullable()
            return nullValue
        elif isinstance(defaultValue, IDLDefaultDictionaryValue):
            assert type.isDictionary()
            return nullValue
        elif isinstance(defaultValue, IDLEmptySequenceValue):
            assert type.isSequence()
            return "Vec::new()"

        raise TypeError("Can't handle non-null, non-empty sequence or non-empty dictionary default value here")

    # A helper function for wrapping up the template body for
    # possibly-nullable objecty stuff
    def wrapObjectTemplate(templateBody, nullValue, isDefinitelyObject, type,
                           failureCode=None):
        if not isDefinitelyObject:
            # Handle the non-object cases by wrapping up the whole
            # thing in an if cascade.
            templateBody = (
                "if ${val}.get().is_object() {\n"
                f"{CGIndenter(CGGeneric(templateBody)).define()}\n")
            if type.nullable():
                templateBody += (
                    "} else if ${val}.get().is_null_or_undefined() {\n"
                    f"    {nullValue}\n")
            templateBody += (
                "} else {\n"
                f"{CGIndenter(onFailureNotAnObject(failureCode)).define()}"
                "}")
        return templateBody

    # A helper function for types that implement FromJSValConvertible trait
    def fromJSValTemplate(config, errorHandler, exceptionCode):
        return f"""match FromJSValConvertible::from_jsval(*cx, ${{val}}, {config}) {{
    Ok(ConversionResult::Success(value)) => value,
    Ok(ConversionResult::Failure(error)) => {{
        {errorHandler}
    }}
    _ => {{
        {exceptionCode}
    }},
}}
"""

    assert not (isEnforceRange and isClamp)  # These are mutually exclusive

    if type.isSequence() or type.isRecord():
        innerInfo = getJSToNativeConversionInfo(innerContainerType(type),
                                                descriptorProvider,
                                                isMember="Sequence",
                                                isAutoRooted=isAutoRooted)
        declType = wrapInNativeContainerType(type, innerInfo.declType)
        config = getConversionConfigForType(type, innerContainerType(type).hasEnforceRange(), isClamp, treatNullAs)

        if type.nullable():
            declType = CGWrapper(declType, pre="Option<", post=" >")

        templateBody = fromJSValTemplate(config, failOrPropagate, exceptionCode)

        return handleOptional(templateBody, declType, handleDefault("None"))

    if type.isUnion():
        declType = CGGeneric(union_native_type(type))
        if type.nullable():
            declType = CGWrapper(declType, pre="Option<", post=" >")

        templateBody = fromJSValTemplate("()", failOrPropagate, exceptionCode)

        dictionaries = [
            memberType
            for memberType in type.unroll().flatMemberTypes
            if memberType.isDictionary()
        ]
        if (defaultValue
                and not isinstance(defaultValue, IDLNullValue)
                and not isinstance(defaultValue, IDLDefaultDictionaryValue)):
            tag = defaultValue.type.tag()
            if tag is IDLType.Tags.bool:
                boolean = "true" if defaultValue.value else "false"
                default = f"{union_native_type(type)}::Boolean({boolean})"
            elif tag is IDLType.Tags.usvstring:
                default = f'{union_native_type(type)}::USVString(USVString("{defaultValue.value}".to_owned()))'
            elif defaultValue.type.isEnum():
                enum = defaultValue.type.inner.identifier.name
                default = f"{union_native_type(type)}::{enum}({enum}::{getEnumValueName(defaultValue.value)})"
            else:
                raise NotImplementedError("We don't currently support default values that aren't \
                                          null, boolean or default dictionary")
        elif dictionaries:
            if defaultValue:
                assert isinstance(defaultValue, IDLDefaultDictionaryValue)
                dictionary, = dictionaries
                default = (
                    f"{union_native_type(type)}::{dictionary.name}("
                    f"{CGDictionary.makeModuleName(dictionary.inner)}::"
                    f"{CGDictionary.makeDictionaryName(dictionary.inner)}::empty())"
                )
            else:
                default = None
        else:
            default = handleDefault("None")

        return handleOptional(templateBody, declType, default)

    if type.isPromise():
        assert not type.nullable()
        # Per spec, what we're supposed to do is take the original
        # Promise.resolve and call it with the original Promise as this
        # value to make a Promise out of whatever value we actually have
        # here.  The question is which global we should use.  There are
        # a couple cases to consider:
        #
        # 1) Normal call to API with a Promise argument.  This is a case the
        #    spec covers, and we should be using the current Realm's
        #    Promise.  That means the current realm.
        # 2) Promise return value from a callback or callback interface.
        #    This is in theory a case the spec covers but in practice it
        #    really doesn't define behavior here because it doesn't define
        #    what Realm we're in after the callback returns, which is when
        #    the argument conversion happens.  We will use the current
        #    realm, which is the realm of the callable (which
        #    may itself be a cross-realm wrapper itself), which makes
        #    as much sense as anything else. In practice, such an API would
        #    once again be providing a Promise to signal completion of an
        #    operation, which would then not be exposed to anyone other than
        #    our own implementation code.
        templateBody = fromJSValTemplate("()", failOrPropagate, exceptionCode)

        if isArgument:
            declType = CGGeneric("&D::Promise")
        else:
            declType = CGGeneric("Rc<D::Promise>")
        return handleOptional(templateBody, declType, handleDefault("None"))

    if type.isGeckoInterface():
        assert not isEnforceRange and not isClamp

        descriptor = descriptorProvider.getDescriptor(
            type.unroll().inner.identifier.name)

        if descriptor.interface.isCallback():
            name = descriptor.nativeType
            declType = CGWrapper(CGGeneric(f"{name}<D>"), pre="Rc<", post=">")
            template = f"{name}::new(cx, ${{val}}.get().to_object())"
            if type.nullable():
                declType = CGWrapper(declType, pre="Option<", post=">")
                template = wrapObjectTemplate(f"Some({template})", "None",
                                              isDefinitelyObject, type,
                                              failureCode)

            return handleOptional(template, declType, handleDefault("None"))

        conversionFunction = "root_from_handlevalue"
        descriptorType = descriptor.returnType
        if isMember == "Variadic":
            conversionFunction = "native_from_handlevalue"
            descriptorType = descriptor.nativeType
        elif isArgument:
            descriptorType = descriptor.argumentType
        elif descriptor.interface.identifier.name == "WindowProxy":
            conversionFunction = "windowproxy_from_handlevalue::<D>"

        if failureCode is None:
            unwrapFailureCode = (
                f'throw_type_error(*cx, "{sourceDescription} does not '
                f'implement interface {descriptor.interface.identifier.name}.");\n'
                f'{exceptionCode}')
        else:
            unwrapFailureCode = failureCode

        templateBody = fill(
            """
            match ${function}($${val}, *cx) {
                Ok(val) => val,
                Err(()) => {
                    $*{failureCode}
                }
            }
            """,
            failureCode=unwrapFailureCode + "\n",
            function=conversionFunction)

        declType = CGGeneric(descriptorType)
        if type.nullable():
            templateBody = f"Some({templateBody})"
            declType = CGWrapper(declType, pre="Option<", post=">")

        templateBody = wrapObjectTemplate(templateBody, "None",
                                          isDefinitelyObject, type, failureCode)

        return handleOptional(templateBody, declType, handleDefault("None"))

    if is_typed_array(type):
        if failureCode is None:
            unwrapFailureCode = (f'throw_type_error(*cx, "{sourceDescription} is not a typed array.");\n'
                                 f'{exceptionCode}')
        else:
            unwrapFailureCode = failureCode

        typeName = type.unroll().name  # unroll because it may be nullable

        if isMember == "Union":
            typeName = f"Heap{typeName}"

        templateBody = fill(
            """
            match typedarray::${ty}::from($${val}.get().to_object()) {
                Ok(val) => val,
                Err(()) => {
                    $*{failureCode}
                }
            }
            """,
            ty=typeName,
            failureCode=f"{unwrapFailureCode}\n",
        )

        if isMember == "Union":
            templateBody = f"RootedTraceableBox::new({templateBody})"

        declType = CGGeneric(f"typedarray::{typeName}")
        if type.nullable():
            templateBody = f"Some({templateBody})"
            declType = CGWrapper(declType, pre="Option<", post=">")

        templateBody = wrapObjectTemplate(templateBody, "None",
                                          isDefinitelyObject, type, failureCode)

        return handleOptional(templateBody, declType, handleDefault("None"))

    elif type.isSpiderMonkeyInterface():
        raise TypeError("Can't handle SpiderMonkey interface arguments other than typed arrays yet")

    if type.isDOMString():
        nullBehavior = getConversionConfigForType(type, isEnforceRange, isClamp, treatNullAs)

        conversionCode = fromJSValTemplate(nullBehavior, failOrPropagate, exceptionCode)

        if defaultValue is None:
            default = None
        elif isinstance(defaultValue, IDLNullValue):
            assert type.nullable()
            default = "None"
        else:
            assert defaultValue.type.tag() == IDLType.Tags.domstring
            default = f'DOMString::from("{defaultValue.value}")'
            if type.nullable():
                default = f"Some({default})"

        declType = "DOMString"
        if type.nullable():
            declType = f"Option<{declType}>"

        return handleOptional(conversionCode, CGGeneric(declType), default)

    if type.isUSVString():
        assert not isEnforceRange and not isClamp

        conversionCode = fromJSValTemplate("()", failOrPropagate, exceptionCode)

        if defaultValue is None:
            default = None
        elif isinstance(defaultValue, IDLNullValue):
            assert type.nullable()
            default = "None"
        else:
            assert defaultValue.type.tag() in (IDLType.Tags.domstring, IDLType.Tags.usvstring)
            default = f'USVString("{defaultValue.value}".to_owned())'
            if type.nullable():
                default = f"Some({default})"

        declType = "USVString"
        if type.nullable():
            declType = f"Option<{declType}>"

        return handleOptional(conversionCode, CGGeneric(declType), default)

    if type.isByteString():
        assert not isEnforceRange and not isClamp

        conversionCode = fromJSValTemplate("()", failOrPropagate, exceptionCode)

        if defaultValue is None:
            default = None
        elif isinstance(defaultValue, IDLNullValue):
            assert type.nullable()
            default = "None"
        else:
            assert defaultValue.type.tag() in (IDLType.Tags.domstring, IDLType.Tags.bytestring)
            default = f'ByteString::new(b"{defaultValue.value}".to_vec())'
            if type.nullable():
                default = f"Some({default})"

        declType = "ByteString"
        if type.nullable():
            declType = f"Option<{declType}>"

        return handleOptional(conversionCode, CGGeneric(declType), default)

    if type.isEnum():
        assert not isEnforceRange and not isClamp

        if type.nullable():
            raise TypeError("We don't support nullable enumerated arguments "
                            "yet")
        enum = type.inner.identifier.name
        if invalidEnumValueFatal:
            handleInvalidEnumValueCode = failureCode or f"throw_type_error(*cx, &error); {exceptionCode}"
        else:
            handleInvalidEnumValueCode = "return true;"

        template = fromJSValTemplate("()", handleInvalidEnumValueCode, exceptionCode)

        if defaultValue is not None:
            assert defaultValue.type.tag() == IDLType.Tags.domstring
            default = f"{enum}::{getEnumValueName(defaultValue.value)}"
        else:
            default = None

        return handleOptional(template, CGGeneric(enum), default)

    if type.isCallback():
        assert not isEnforceRange and not isClamp
        assert not type.treatNonCallableAsNull()
        assert not type.treatNonObjectAsNull() or type.nullable()
        assert not type.treatNonObjectAsNull() or not type.treatNonCallableAsNull()

        callback = type.unroll().callback
        declType = CGGeneric(f"{callback.identifier.name}<D>")
        finalDeclType = CGTemplatedType("Rc", declType)

        conversion = CGCallbackTempRoot(declType.define())

        if type.nullable():
            declType = CGTemplatedType("Option", declType)
            finalDeclType = CGTemplatedType("Option", finalDeclType)
            conversion = CGWrapper(conversion, pre="Some(", post=")")

        if allowTreatNonObjectAsNull and type.treatNonObjectAsNull():
            if not isDefinitelyObject:
                haveObject = "${val}.get().is_object()"
                template = CGIfElseWrapper(haveObject,
                                           conversion,
                                           CGGeneric("None")).define()
            else:
                template = conversion
        else:
            template = CGIfElseWrapper("IsCallable(${val}.get().to_object())",
                                       conversion,
                                       onFailureNotCallable(failureCode)).define()
            template = wrapObjectTemplate(
                template,
                "None",
                isDefinitelyObject,
                type,
                failureCode)

        if defaultValue is not None:
            assert allowTreatNonObjectAsNull
            assert type.treatNonObjectAsNull()
            assert type.nullable()
            assert isinstance(defaultValue, IDLNullValue)
            default = "None"
        else:
            default = None

        return JSToNativeConversionInfo(template, default, finalDeclType)

    if type.isAny():
        assert not isEnforceRange and not isClamp
        assert isMember != "Union"

        if isMember in ("Dictionary", "Sequence") or isAutoRooted:
            templateBody = "${val}.get()"

            if defaultValue is None:
                default = None
            elif isinstance(defaultValue, IDLNullValue):
                default = "NullValue()"
            elif isinstance(defaultValue, IDLUndefinedValue):
                default = "UndefinedValue()"
            else:
                raise TypeError("Can't handle non-null, non-undefined default value here")

            if not isAutoRooted:
                templateBody = f"RootedTraceableBox::from_box(Heap::boxed({templateBody}))"
                if default is not None:
                    default = f"RootedTraceableBox::from_box(Heap::boxed({default}))"
                declType = CGGeneric("RootedTraceableBox<Heap<JSVal>>")
            # AutoRooter can trace properly inner raw GC thing pointers
            else:
                declType = CGGeneric("JSVal")

            return handleOptional(templateBody, declType, default)

        declType = CGGeneric("HandleValue")

        if defaultValue is None:
            default = None
        elif isinstance(defaultValue, IDLNullValue):
            default = "HandleValue::null()"
        elif isinstance(defaultValue, IDLUndefinedValue):
            default = "HandleValue::undefined()"
        else:
            raise TypeError("Can't handle non-null, non-undefined default value here")

        return handleOptional("${val}", declType, default)

    if type.isObject():
        assert not isEnforceRange and not isClamp

        templateBody = "${val}.get().to_object()"
        default = "ptr::null_mut()"

        if isMember in ("Dictionary", "Union", "Sequence") and not isAutoRooted:
            templateBody = f"RootedTraceableBox::from_box(Heap::boxed({templateBody}))"
            default = "RootedTraceableBox::new(Heap::default())"
            declType = CGGeneric("RootedTraceableBox<Heap<*mut JSObject>>")
        else:
            # TODO: Need to root somehow
            # https://github.com/servo/servo/issues/6382
            declType = CGGeneric("*mut JSObject")

        templateBody = wrapObjectTemplate(templateBody, default,
                                          isDefinitelyObject, type, failureCode)

        return handleOptional(templateBody, declType,
                              handleDefault(default))

    if type.isDictionary():
        # There are no nullable dictionaries
        assert not type.nullable() or (isMember and isMember != "Dictionary")

        typeName = f"{CGDictionary.makeModuleName(type.inner)}::{CGDictionary.makeDictionaryName(type.inner)}"
        if containsDomInterface(type):
            typeName += "<D>"
        declType = CGGeneric(typeName)
        empty = f"{typeName.replace('<D>', '')}::empty()"

        if type_needs_tracing(type):
            declType = CGTemplatedType("RootedTraceableBox", declType)

        template = fromJSValTemplate("()", failOrPropagate, exceptionCode)

        return handleOptional(template, declType, handleDefault(empty))

    if type.isUndefined():
        # This one only happens for return values, and its easy: Just
        # ignore the jsval.
        return JSToNativeConversionInfo("", None, None)

    if not type.isPrimitive():
        raise TypeError(f"Need conversion for argument type '{type}'")

    conversionBehavior = getConversionConfigForType(type, isEnforceRange, isClamp, treatNullAs)

    if failureCode is None:
        failureCode = 'return false'

    declType = CGGeneric(builtinNames[type.tag()])
    if type.nullable():
        declType = CGWrapper(declType, pre="Option<", post=">")

    template = fromJSValTemplate(conversionBehavior, failOrPropagate, exceptionCode)

    if defaultValue is not None:
        if isinstance(defaultValue, IDLNullValue):
            assert type.nullable()
            defaultStr = "None"
        else:
            tag = defaultValue.type.tag()
            if tag in [IDLType.Tags.float, IDLType.Tags.double]:
                defaultStr = f"Finite::wrap({defaultValue.value})"
            elif tag in numericTags:
                defaultStr = str(defaultValue.value)
            else:
                assert tag == IDLType.Tags.bool
                defaultStr = toStringBool(defaultValue.value)

            if type.nullable():
                defaultStr = f"Some({defaultStr})"
    else:
        defaultStr = None

    return handleOptional(template, declType, defaultStr)


def wrapForType(jsvalRef, result='result', successCode='true', pre=''):
    """
    Reflect a Rust value into JS.

      * 'jsvalRef': a MutableHandleValue in which to store the result
                    of the conversion;
      * 'result': the name of the variable in which the Rust value is stored;
      * 'successCode': the code to run once we have done the conversion.
      * 'pre': code to run before the conversion if rooting is necessary
    """
    wrap = f"{pre}\n({result}).to_jsval(*cx, {jsvalRef});"
    if successCode:
        wrap += f"\n{successCode}"
    return wrap


def getUnionTypeTemplateVars(type, descriptorProvider: DescriptorProvider):
    if type.isGeckoInterface():
        name = type.inner.identifier.name
        typeName = descriptorProvider.getDescriptor(name).returnType
    elif type.isEnum():
        name = type.inner.identifier.name
        typeName = name
    elif type.isDictionary():
        name = type.name
        typeName = name
        if containsDomInterface(type):
            typeName += "<D>"
    elif type.isSequence() or type.isRecord():
        name = type.name
        inner = getUnionTypeTemplateVars(innerContainerType(type), descriptorProvider)
        typeName = wrapInNativeContainerType(type, CGGeneric(inner["typeName"])).define()
    elif type.isByteString():
        name = type.name
        typeName = "ByteString"
    elif type.isDOMString():
        name = type.name
        typeName = "DOMString"
    elif type.isUSVString():
        name = type.name
        typeName = "USVString"
    elif type.isPrimitive():
        name = type.name
        typeName = builtinNames[type.tag()]
    elif type.isObject():
        name = type.name
        typeName = "Heap<*mut JSObject>"
    elif is_typed_array(type):
        name = type.name
        typeName = f"typedarray::Heap{name}"
    elif type.isCallback():
        name = type.name
        typeName = f"{name}<D>"
    elif type.isUndefined():
        return {
            "name": type.name,
            "typeName": "()",
            "jsConversion": CGGeneric("if value.is_undefined() { Ok(Some(())) } else { Ok(None) }")
        }
    else:
        raise TypeError(f"Can't handle {type} in unions yet")

    info = getJSToNativeConversionInfo(
        type, descriptorProvider, failureCode="return Ok(None);",
        exceptionCode='return Err(());',
        isDefinitelyObject=True,
        isMember="Union")
    template = info.template

    jsConversion = string.Template(template).substitute({
        "val": "value",
    })
    jsConversion = CGWrapper(CGGeneric(jsConversion), pre="Ok(Some(", post="))")

    return {
        "name": name,
        "typeName": typeName,
        "jsConversion": jsConversion,
    }


def type_needs_tracing(t):
    assert isinstance(t, IDLObject), (t, type(t))

    if t.isType():
        if isinstance(t, IDLWrapperType):
            return type_needs_tracing(t.inner)

        if t.nullable():
            return type_needs_tracing(t.inner)

        if t.isAny():
            return True

        if t.isObject():
            return True

        if t.isSequence():
            return type_needs_tracing(t.inner)

        if t.isUnion():
            return any(type_needs_tracing(member) for member in t.flatMemberTypes)

        if is_typed_array(t):
            return True

        return False

    if t.isDictionary():
        if t.parent and type_needs_tracing(t.parent):
            return True

        if any(type_needs_tracing(member.type) for member in t.members):
            return True

        return False

    if t.isInterface():
        return False

    if t.isEnum():
        return False

    assert False, (t, type(t))


def argument_type(descriptorProvider, ty, optional=False, defaultValue=None, variadic=False):
    info = getJSToNativeConversionInfo(
        ty, descriptorProvider, isArgument=True,
        isAutoRooted=type_needs_auto_root(ty))
    declType = info.declType

    if variadic:
        if ty.isGeckoInterface():
            declType = CGWrapper(declType, pre="&[", post="]")
        else:
            declType = CGWrapper(declType, pre="Vec<", post=">")
    elif optional and not defaultValue:
        declType = CGWrapper(declType, pre="Option<", post=">")

    if ty.isDictionary() and not type_needs_tracing(ty):
        declType = CGWrapper(declType, pre="&")

    if type_needs_auto_root(ty):
        declType = CGTemplatedType("CustomAutoRooterGuard", declType)

    return declType.define()


def return_type(descriptorProvider, rettype, infallible):
    result = getRetvalDeclarationForType(rettype, descriptorProvider)
    if rettype and returnTypeNeedsOutparam(rettype):
        result = CGGeneric("()")
    if not infallible:
        result = CGWrapper(result, pre="Fallible<", post=">")
    return result.define()
