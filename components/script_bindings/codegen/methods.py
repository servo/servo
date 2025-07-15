from components.script_bindings.codegen.base import CGThing, CGList, CGWrapper, CGGeneric, CGAbstractExternMethod
from components.script_bindings.codegen.codegen import CGCase, CGArgumentConverter, CGIndenter, \
    instantiateJSToNativeConversionTemplate, type_needs_auto_root, CGSwitch, CGMaplikeOrSetlikeMethodGenerator, \
    CGIterableMethodGenerator, CGCallGenerator, returnTypeNeedsOutparam, FakeArgument, Argument, \
    CGSpecializedGetter, CGAbstractStaticBindingMethod
from components.script_bindings.codegen.configuration import MakeNativeName
from components.script_bindings.codegen.types import getJSToNativeConversionInfo, wrapForType


class CGMethodCall(CGThing):
    """
    A class to generate selection of a method signature from a set of
    signatures and generation of a call to that signature.
    """
    def __init__(self, argsPre, nativeMethodName, static, descriptor, method):
        CGThing.__init__(self)

        methodName = f'\\"{descriptor.interface.identifier.name}.{method.identifier.name}\\"'

        def requiredArgCount(signature):
            arguments = signature[1]
            if len(arguments) == 0:
                return 0
            requiredArgs = len(arguments)
            while requiredArgs and arguments[requiredArgs - 1].optional:
                requiredArgs -= 1
            return requiredArgs

        signatures = method.signatures()

        def getPerSignatureCall(signature, argConversionStartsAt=0):
            signatureIndex = signatures.index(signature)
            return CGPerSignatureCall(signature[0], argsPre, signature[1],
                                      f"{nativeMethodName}{'_' * signatureIndex}",
                                      static, descriptor,
                                      method, argConversionStartsAt)

        if len(signatures) == 1:
            # Special case: we can just do a per-signature method call
            # here for our one signature and not worry about switching
            # on anything.
            signature = signatures[0]
            self.cgRoot = CGList([getPerSignatureCall(signature)])
            requiredArgs = requiredArgCount(signature)

            if requiredArgs > 0:
                code = (
                    f"if argc < {requiredArgs} {{\n"
                    f"    throw_type_error(*cx, \"Not enough arguments to {methodName}.\");\n"
                    "    return false;\n"
                    "}")
                self.cgRoot.prepend(
                    CGWrapper(CGGeneric(code), pre="\n", post="\n"))

            return

        # Need to find the right overload
        maxArgCount = method.maxArgCount
        allowedArgCounts = method.allowedArgCounts

        argCountCases = []
        for argCount in allowedArgCounts:
            possibleSignatures = method.signaturesForArgCount(argCount)
            if len(possibleSignatures) == 1:
                # easy case!
                signature = possibleSignatures[0]
                argCountCases.append(CGCase(str(argCount), getPerSignatureCall(signature)))
                continue

            distinguishingIndex = method.distinguishingIndexForArgCount(argCount)

            # We can't handle unions of non-object values at the distinguishing index.
            for (returnType, args) in possibleSignatures:
                type = args[distinguishingIndex].type
                if type.isUnion():
                    if type.nullable():
                        type = type.inner
                    for type in type.flatMemberTypes:
                        if not (type.isObject() or type.isNonCallbackInterface()):
                            raise TypeError("No support for unions with non-object variants "
                                            f"as distinguishing arguments yet: {args[distinguishingIndex].location}",
                                            )

            # Convert all our arguments up to the distinguishing index.
            # Doesn't matter which of the possible signatures we use, since
            # they all have the same types up to that point; just use
            # possibleSignatures[0]
            caseBody = [
                CGArgumentConverter(possibleSignatures[0][1][i],
                                    i, "args", "argc", descriptor)
                for i in range(0, distinguishingIndex)]

            # Select the right overload from our set.
            distinguishingArg = f"HandleValue::from_raw(args.get({distinguishingIndex}))"

            def pickFirstSignature(condition, filterLambda):
                sigs = list(filter(filterLambda, possibleSignatures))
                assert len(sigs) < 2
                if len(sigs) > 0:
                    call = getPerSignatureCall(sigs[0], distinguishingIndex)
                    if condition is None:
                        caseBody.append(call)
                    else:
                        caseBody.append(CGGeneric(f"if {condition} {{"))
                        caseBody.append(CGIndenter(call))
                        caseBody.append(CGGeneric("}"))
                    return True
                return False

            # First check for null or undefined
            pickFirstSignature(f"{distinguishingArg}.get().is_null_or_undefined()",
                               lambda s: (s[1][distinguishingIndex].type.nullable()
                                          or s[1][distinguishingIndex].type.isDictionary()))

            # Now check for distinguishingArg being an object that implements a
            # non-callback interface.  That includes typed arrays and
            # arraybuffers.
            interfacesSigs = [
                s for s in possibleSignatures
                if (s[1][distinguishingIndex].type.isObject()
                    or s[1][distinguishingIndex].type.isUnion()
                    or s[1][distinguishingIndex].type.isNonCallbackInterface())]
            # There might be more than one of these; we need to check
            # which ones we unwrap to.

            if len(interfacesSigs) > 0:
                # The spec says that we should check for "platform objects
                # implementing an interface", but it's enough to guard on these
                # being an object.  The code for unwrapping non-callback
                # interfaces and typed arrays will just bail out and move on to
                # the next overload if the object fails to unwrap correctly.  We
                # could even not do the isObject() check up front here, but in
                # cases where we have multiple object overloads it makes sense
                # to do it only once instead of for each overload.  That will
                # also allow the unwrapping test to skip having to do codegen
                # for the null-or-undefined case, which we already handled
                # above.
                caseBody.append(CGGeneric(f"if {distinguishingArg}.get().is_object() {{"))
                for idx, sig in enumerate(interfacesSigs):
                    caseBody.append(CGIndenter(CGGeneric("'_block: {")))
                    type = sig[1][distinguishingIndex].type

                    # The argument at index distinguishingIndex can't possibly
                    # be unset here, because we've already checked that argc is
                    # large enough that we can examine this argument.
                    info = getJSToNativeConversionInfo(
                        type, descriptor, failureCode="break '_block;", isDefinitelyObject=True)
                    template = info.template
                    declType = info.declType

                    testCode = instantiateJSToNativeConversionTemplate(
                        template,
                        {"val": distinguishingArg},
                        declType,
                        f"arg{distinguishingIndex}",
                        needsAutoRoot=type_needs_auto_root(type))

                    # Indent by 4, since we need to indent further than our "do" statement
                    caseBody.append(CGIndenter(testCode, 4))
                    # If we got this far, we know we unwrapped to the right
                    # interface, so just do the call.  Start conversion with
                    # distinguishingIndex + 1, since we already converted
                    # distinguishingIndex.
                    caseBody.append(CGIndenter(
                        getPerSignatureCall(sig, distinguishingIndex + 1), 4))
                    caseBody.append(CGIndenter(CGGeneric("}")))

                caseBody.append(CGGeneric("}"))

            # XXXbz Now we're supposed to check for distinguishingArg being
            # an array or a platform object that supports indexed
            # properties... skip that last for now.  It's a bit of a pain.
            pickFirstSignature(f"{distinguishingArg}.get().is_object() && is_array_like::<D>(*cx, {distinguishingArg})",
                               lambda s:
                                   (s[1][distinguishingIndex].type.isSequence()
                                    or s[1][distinguishingIndex].type.isObject()))

            # Check for vanilla JS objects
            # XXXbz Do we need to worry about security wrappers?
            pickFirstSignature(f"{distinguishingArg}.get().is_object()",
                               lambda s: (s[1][distinguishingIndex].type.isCallback()
                                          or s[1][distinguishingIndex].type.isCallbackInterface()
                                          or s[1][distinguishingIndex].type.isDictionary()
                                          or s[1][distinguishingIndex].type.isObject()))

            # The remaining cases are mutually exclusive.  The
            # pickFirstSignature calls are what change caseBody
            # Check for strings or enums
            if pickFirstSignature(None,
                                  lambda s: (s[1][distinguishingIndex].type.isString()
                                             or s[1][distinguishingIndex].type.isEnum())):
                pass
            # Check for primitives
            elif pickFirstSignature(None,
                                    lambda s: s[1][distinguishingIndex].type.isPrimitive()):
                pass
            # Check for "any"
            elif pickFirstSignature(None,
                                    lambda s: s[1][distinguishingIndex].type.isAny()):
                pass
            else:
                # Just throw; we have no idea what we're supposed to
                # do with this.
                caseBody.append(CGGeneric("throw_type_error(*cx, \"Could not convert JavaScript argument\");\n"
                                          "return false;"))

            argCountCases.append(CGCase(str(argCount),
                                        CGList(caseBody, "\n")))

        overloadCGThings = []
        overloadCGThings.append(
            CGGeneric(f"let argcount = cmp::min(argc, {maxArgCount});"))
        overloadCGThings.append(
            CGSwitch("argcount",
                     argCountCases,
                     CGGeneric(f"throw_type_error(*cx, \"Not enough arguments to {methodName}.\");\n"
                               "return false;")))
        # XXXjdm Avoid unreachable statement warnings
        # overloadCGThings.append(
        #     CGGeneric('panic!("We have an always-returning default case");\n'
        #               'return false;'))
        self.cgRoot = CGWrapper(CGList(overloadCGThings, "\n"),
                                pre="\n")

    def define(self):
        return self.cgRoot.define()


class CGPerSignatureCall(CGThing):
    """
    This class handles the guts of generating code for a particular
    call signature.  A call signature consists of four things:

    1) A return type, which can be None to indicate that there is no
       actual return value (e.g. this is an attribute setter) or an
       IDLType if there's an IDL type involved (including |void|).
    2) An argument list, which is allowed to be empty.
    3) A name of a native method to call.
    4) Whether or not this method is static.

    We also need to know whether this is a method or a getter/setter
    to do error reporting correctly.

    The idlNode parameter can be either a method or an attr. We can query
    |idlNode.identifier| in both cases, so we can be agnostic between the two.
    """
    # XXXbz For now each entry in the argument list is either an
    # IDLArgument or a FakeArgument, but longer-term we may want to
    # have ways of flagging things like JSContext* or optional_argc in
    # there.

    def __init__(self, returnType, argsPre, arguments, nativeMethodName, static,
                 descriptor, idlNode, argConversionStartsAt=0,
                 getter=False, setter=False):
        CGThing.__init__(self)
        self.returnType = returnType
        self.descriptor = descriptor
        self.idlNode = idlNode
        self.extendedAttributes = descriptor.getExtendedAttributes(idlNode,
                                                                   getter=getter,
                                                                   setter=setter)
        self.argsPre = argsPre
        self.arguments = arguments
        self.argCount = len(arguments)
        cgThings = []
        cgThings.extend([CGArgumentConverter(arguments[i], i, self.getArgs(),
                                             self.getArgc(), self.descriptor,
                                             invalidEnumValueFatal=not setter) for
                         i in range(argConversionStartsAt, self.argCount)])

        errorResult = None
        if self.isFallible():
            errorResult = " false"

        if idlNode.isMethod() and idlNode.isMaplikeOrSetlikeOrIterableMethod():
            if idlNode.maplikeOrSetlikeOrIterable.isMaplike() or \
               idlNode.maplikeOrSetlikeOrIterable.isSetlike():
                cgThings.append(CGMaplikeOrSetlikeMethodGenerator(descriptor,
                                                                  idlNode.maplikeOrSetlikeOrIterable,
                                                                  idlNode.identifier.name))
            else:
                cgThings.append(CGIterableMethodGenerator(descriptor,
                                                          idlNode.maplikeOrSetlikeOrIterable,
                                                          idlNode.identifier.name))
        else:
            hasCEReactions = idlNode.getExtendedAttribute("CEReactions")
            cgThings.append(CGCallGenerator(
                errorResult,
                self.getArguments(), self.argsPre, returnType,
                self.extendedAttributes, descriptor, nativeMethodName,
                static, hasCEReactions=hasCEReactions))

        self.cgRoot = CGList(cgThings, "\n")

    def getArgs(self):
        return "args" if self.argCount > 0 else ""

    def getArgc(self):
        return "argc"

    def getArguments(self):
        return [(a, process_arg(f"arg{i}", a)) for (i, a) in enumerate(self.arguments)]

    def isFallible(self):
        return 'infallible' not in self.extendedAttributes

    def wrap_return_value(self):
        resultName = "result"
        # Maplike methods have `any` return values in WebIDL, but our internal bindings
        # use stronger types so we need to exclude them from being handled like other
        # generated code.
        if returnTypeNeedsOutparam(self.returnType) and (
           not (self.idlNode.isMethod() and self.idlNode.isMaplikeOrSetlikeOrIterableMethod())):
            resultName = "retval"
        return wrapForType(
            'MutableHandleValue::from_raw(args.rval())',
            result=resultName,
            successCode='return true;',
        )

    def define(self):
        return f"{self.cgRoot.define()}\n{self.wrap_return_value()}"


class CGGetterCall(CGPerSignatureCall):
    """
    A class to generate a native object getter call for a particular IDL
    getter.
    """
    def __init__(self, argsPre, returnType, nativeMethodName, descriptor, attr):
        CGPerSignatureCall.__init__(self, returnType, argsPre, [],
                                    nativeMethodName, attr.isStatic(), descriptor,
                                    attr, getter=True)


class CGSetterCall(CGPerSignatureCall):
    """
    A class to generate a native object setter call for a particular IDL
    setter.
    """
    def __init__(self, argsPre, argType, nativeMethodName, descriptor, attr):
        CGPerSignatureCall.__init__(self, None, argsPre,
                                    [FakeArgument(argType, attr, allowTreatNonObjectAsNull=True)],
                                    nativeMethodName, attr.isStatic(), descriptor, attr,
                                    setter=True)

    def wrap_return_value(self):
        # We have no return value
        return "\ntrue"

    def getArgc(self):
        return "1"


class CGSpecializedMethod(CGAbstractExternMethod):
    """
    A class for generating the C++ code for a specialized method that the JIT
    can call with lower overhead.
    """
    def __init__(self, descriptor, method):
        self.method = method
        name = method.identifier.name
        args = [Argument('*mut JSContext', 'cx'),
                Argument('RawHandleObject', '_obj'),
                Argument('*mut libc::c_void', 'this'),
                Argument('*const JSJitMethodCallArgs', 'args')]
        CGAbstractExternMethod.__init__(self, descriptor, name, 'bool', args, templateArgs=["D: DomTypes"])

    def definition_body(self):
        nativeName = CGSpecializedMethod.makeNativeName(self.descriptor,
                                                        self.method)
        return CGWrapper(CGMethodCall([], nativeName, self.method.isStatic(),
                                      self.descriptor, self.method),
                         pre="let cx = SafeJSContext::from_ptr(cx);\n"
                             f"let this = &*(this as *const {self.descriptor.concreteType});\n"
                             "let args = &*args;\n"
                             "let argc = args.argc_;\n")

    @staticmethod
    def makeNativeName(descriptor, method):
        if method.underlyingAttr:
            return CGSpecializedGetter.makeNativeName(descriptor, method.underlyingAttr)
        name = method.identifier.name
        nativeName = descriptor.binaryNameFor(name, method.isStatic())
        if nativeName == name:
            nativeName = descriptor.internalNameFor(name)
        return MakeNativeName(nativeName)


class CGStaticMethod(CGAbstractStaticBindingMethod):
    """
    A class for generating the Rust code for an IDL static method.
    """
    def __init__(self, descriptor, method):
        self.method = method
        name = descriptor.binaryNameFor(method.identifier.name, True)
        CGAbstractStaticBindingMethod.__init__(self, descriptor, name, templateArgs=["D: DomTypes"])

    def generate_code(self):
        nativeName = CGSpecializedMethod.makeNativeName(self.descriptor,
                                                        self.method)
        safeContext = CGGeneric("let cx = SafeJSContext::from_ptr(cx);\n")
        setupArgs = CGGeneric("let args = CallArgs::from_vp(vp, argc);\n")
        call = CGMethodCall(["&global"], nativeName, True, self.descriptor, self.method)
        return CGList([safeContext, setupArgs, call])


def process_arg(expr, arg):
    if arg.type.isGeckoInterface() and not arg.type.unroll().inner.isCallback():
        if arg.variadic or arg.type.isSequence():
            expr += ".r()"
        elif arg.type.nullable() and arg.optional and not arg.defaultValue:
            expr += ".as_ref().map(Option::as_deref)"
        elif arg.type.nullable() or arg.optional and not arg.defaultValue:
            expr += ".as_deref()"
        else:
            expr = f"&{expr}"
    elif isinstance(arg.type, IDLPromiseType):
        expr = f"&{expr}"
    return expr
