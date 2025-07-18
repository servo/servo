import re

from components.script_bindings.codegen.base import CGGeneric, CGList
from components.script_bindings.codegen.codegen import CGClass, ClassBase, ClassConstructor, Argument, ClassMethod, \
    CGNativeMember, returnTypeNeedsOutparam, FakeMember, instantiateJSToNativeConversionTemplate, CGIndenter, \
    CGIfWrapper, CGIfElseWrapper
from components.script_bindings.codegen.configuration import MakeNativeName
from components.script_bindings.codegen.types import getJSToNativeConversionInfo, wrapForType


class CGCallback(CGClass):
    def __init__(self, idlObject, descriptorProvider, baseName, methods):
        self.baseName = baseName
        self._deps = idlObject.getDeps()
        name = idlObject.identifier.name
        # For our public methods that needThisHandling we want most of the
        # same args and the same return type as what CallbackMember
        # generates.  So we want to take advantage of all its
        # CGNativeMember infrastructure, but that infrastructure can't deal
        # with templates and most especially template arguments.  So just
        # cheat and have CallbackMember compute all those things for us.
        realMethods = []
        for method in methods:
            if not method.needThisHandling:
                realMethods.append(method)
            else:
                realMethods.extend(self.getMethodImpls(method))
        CGClass.__init__(self, name,
                         bases=[ClassBase(baseName)],
                         constructors=self.getConstructors(),
                         methods=realMethods,
                         templateSpecialization=['D: DomTypes'],
                         decorators="#[derive(JSTraceable, PartialEq)]\n"
                                    "#[cfg_attr(crown, allow(crown::unrooted_must_root))]\n"
                                    "#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]")

    def getConstructors(self):
        return [ClassConstructor(
            [Argument("SafeJSContext", "aCx"), Argument("*mut JSObject", "aCallback")],
            bodyInHeader=True,
            visibility="pub",
            explicit=False,
            baseConstructors=[
                f"{self.baseName.replace('<D>', '')}::new()"
            ])]

    def getMethodImpls(self, method):
        assert method.needThisHandling
        args = list(method.args)
        # Strip out the JSContext*/JSObject* args
        # that got added.
        assert args[0].name == "cx" and args[0].argType == "SafeJSContext"
        assert args[1].name == "aThisObj" and args[1].argType == "HandleValue"
        args = args[2:]
        # Record the names of all the arguments, so we can use them when we call
        # the private method.
        argnames = [arg.name for arg in args] + ["can_gc"]
        argnamesWithThis = ["s.get_context()", "thisValue.handle()"] + argnames
        argnamesWithoutThis = ["s.get_context()", "HandleValue::undefined()"] + argnames
        # Now that we've recorded the argnames for our call to our private
        # method, insert our optional argument for deciding whether the
        # CallSetup should re-throw exceptions on aRv.
        args.append(Argument("ExceptionHandling", "aExceptionHandling",
                             "ReportExceptions"))

        args.append(Argument("CanGc", "can_gc"))
        method.args.append(Argument("CanGc", "can_gc"))

        # And now insert our template argument.
        argsWithoutThis = list(args)
        args.insert(0, Argument("&T", "thisObj"))

        # And the self argument
        method.args.insert(0, Argument(None, "&self"))
        args.insert(0, Argument(None, "&self"))
        argsWithoutThis.insert(0, Argument(None, "&self"))

        setupCall = "let s = CallSetup::<D>::new(self, aExceptionHandling);\n"

        bodyWithThis = (
            f"{setupCall}rooted!(in(*s.get_context()) let mut thisValue: JSVal);\n"
            "let wrap_result = wrap_call_this_value(s.get_context(), thisObj, thisValue.handle_mut());\n"
            "if !wrap_result {\n"
            "    return Err(JSFailed);\n"
            "}\n"
            f"unsafe {{ self.{method.name}({', '.join(argnamesWithThis)}) }}")
        bodyWithoutThis = (
            f"{setupCall}\n"
            f"unsafe {{ self.{method.name}({', '.join(argnamesWithoutThis)}) }}")
        return [ClassMethod(f'{method.name}_', method.returnType, args,
                            bodyInHeader=True,
                            templateArgs=["T: ThisReflector"],
                            body=bodyWithThis,
                            visibility='pub'),
                ClassMethod(f'{method.name}__', method.returnType, argsWithoutThis,
                            bodyInHeader=True,
                            body=bodyWithoutThis,
                            visibility='pub'),
                method]

    def deps(self):
        return self._deps


class CGCallbackFunction(CGCallback):
    def __init__(self, callback, descriptorProvider):
        CGCallback.__init__(self, callback, descriptorProvider,
                            "CallbackFunction<D>",
                            methods=[CallCallback(callback, descriptorProvider)])

    def getConstructors(self):
        return CGCallback.getConstructors(self)


class CGCallbackFunctionImpl(CGGeneric):
    def __init__(self, callback):
        type = f"{callback.identifier.name}<D>"
        impl = (f"""
impl<D: DomTypes> CallbackContainer<D> for {type} {{
    unsafe fn new(cx: SafeJSContext, callback: *mut JSObject) -> Rc<{type}> {{
        {type.replace('<D>', '')}::new(cx, callback)
    }}

    fn callback_holder(&self) -> &CallbackObject<D> {{
        self.parent.callback_holder()
    }}
}}

impl<D: DomTypes> ToJSValConvertible for {type} {{
    unsafe fn to_jsval(&self, cx: *mut JSContext, rval: MutableHandleValue) {{
        self.callback().to_jsval(cx, rval);
    }}
}}
""")
        CGGeneric.__init__(self, impl)


class CGCallbackInterface(CGCallback):
    def __init__(self, descriptor):
        iface = descriptor.interface
        attrs = [m for m in iface.members if m.isAttr() and not m.isStatic()]
        assert not attrs
        methods = [m for m in iface.members
                   if m.isMethod() and not m.isStatic() and not m.isIdentifierLess()]
        methods = [CallbackOperation(m, sig, descriptor) for m in methods
                   for sig in m.signatures()]
        assert not iface.isJSImplemented() or not iface.ctor()
        CGCallback.__init__(self, iface, descriptor, "CallbackInterface<D>", methods)


class CallbackMember(CGNativeMember):
    def __init__(self, sig, name, descriptorProvider, needThisHandling):
        """
        needThisHandling is True if we need to be able to accept a specified
        thisObj, False otherwise.
        """

        self.retvalType = sig[0]
        self.originalSig = sig
        args = sig[1]
        self.argCount = len(args)
        if self.argCount > 0:
            # Check for variadic arguments
            lastArg = args[self.argCount - 1]
            if lastArg.variadic:
                self.argCountStr = (
                    f"{self.argCount - 1} + {lastArg.identifier.name}.len()").removeprefix("0 + ")
            else:
                self.argCountStr = f"{self.argCount}"
        self.usingOutparam = returnTypeNeedsOutparam(self.retvalType)
        self.needThisHandling = needThisHandling
        # If needThisHandling, we generate ourselves as private and the caller
        # will handle generating public versions that handle the "this" stuff.
        visibility = "priv" if needThisHandling else "pub"
        # We don't care, for callback codegen, whether our original member was
        # a method or attribute or whatnot.  Just always pass FakeMember()
        # here.
        CGNativeMember.__init__(self, descriptorProvider, FakeMember(),
                                name, (self.retvalType, args),
                                extendedAttrs={},
                                passJSBitsAsNeeded=False,
                                unsafe=needThisHandling,
                                visibility=visibility)
        # We have to do all the generation of our body now, because
        # the caller relies on us throwing if we can't manage it.
        self.exceptionCode = "return Err(JSFailed);\n"
        self.body = self.getImpl()

    def getImpl(self):
        argvDecl = (
            "rooted_vec!(let mut argv);\n"
            f"argv.extend((0..{self.argCountStr}).map(|_| Heap::default()));\n"
        ) if self.argCount > 0 else ""  # Avoid weird 0-sized arrays

        # Newlines and semicolons are in the values
        pre = (
            f"{self.getCallSetup()}"
            f"{self.getRvalDecl()}"
            f"{argvDecl}"
        )
        body = (
            f"{self.getArgConversions()}"
            f"{self.getCall()}"
            f"{self.getResultConversion()}"
        )
        return f"{pre}\n{body}"

    def getResultConversion(self):
        replacements = {
            "val": "rval.handle()",
        }

        info = getJSToNativeConversionInfo(
            self.retvalType,
            self.descriptorProvider,
            exceptionCode=self.exceptionCode,
            isCallbackReturnValue="Callback",
            # XXXbz we should try to do better here
            sourceDescription="return value")
        template = info.template
        declType = info.declType

        if self.usingOutparam:
            convertType = CGGeneric("")
        else:
            convertType = instantiateJSToNativeConversionTemplate(
                template, replacements, declType, "retval")

        if self.retvalType is None or self.retvalType.isUndefined() or self.usingOutparam:
            retval = "()"
        else:
            retval = "retval"

        return f"{convertType.define()}\nOk({retval})\n"

    def getArgConversions(self):
        # Just reget the arglist from self.originalSig, because our superclasses
        # just have way to many members they like to clobber, so I can't find a
        # safe member name to store it in.
        arglist = self.originalSig[1]
        argConversions = [self.getArgConversion(i, arg) for (i, arg)
                          in enumerate(arglist)]
        # Do them back to front, so our argc modifications will work
        # correctly, because we examine trailing arguments first.
        argConversions.reverse()
        argConversions = [CGGeneric(c) for c in argConversions]
        # If arg count is only 1 but it's optional and not default value,
        # argc should be mutable.
        if self.argCount == 1 and not (arglist[0].optional and not arglist[0].defaultValue):
            argConversions.insert(0, self.getArgcDecl(True))
        elif self.argCount > 0:
            argConversions.insert(0, self.getArgcDecl(False))
        # And slap them together.
        return CGList(argConversions, "\n\n").define() + "\n\n"

    def getArgConversion(self, i, arg):
        argval = arg.identifier.name

        if arg.variadic:
            argval = f"{argval}[idx].get()"
            jsvalIndex = f"{i} + idx"
        else:
            jsvalIndex = f"{i}"
            if arg.optional and not arg.defaultValue:
                argval += ".unwrap()"

        conversion = wrapForType(
            "argv_root.handle_mut()", result=argval,
            successCode=("{\n"
                         f"let arg = &mut argv[{jsvalIndex.removeprefix('0 + ')}];\n"
                         "*arg = Heap::default();\n"
                         "arg.set(argv_root.get());\n"
                         "}"),
            pre="rooted!(in(*cx) let mut argv_root = UndefinedValue());")
        if arg.variadic:
            conversion = (
                f"for idx in 0..{arg.identifier.name}.len() {{\n"
                f"{CGIndenter(CGGeneric(conversion)).define()}\n}}"
            )
        elif arg.optional and not arg.defaultValue:
            conversion = (
                f"{CGIfWrapper(f'{arg.identifier.name}.is_some()', CGGeneric(conversion)).define()}"
                f" else if argc == {i + 1} {{\n"
                "    // This is our current trailing argument; reduce argc\n"
                "    argc -= 1;\n"
                "} else {\n"
                f"    argv[{i}] = Heap::default();\n"
                "}"
            )
        return conversion

    def getArgs(self, returnType, argList):
        args = CGNativeMember.getArgs(self, returnType, argList)
        if not self.needThisHandling:
            # Since we don't need this handling, we're the actual method that
            # will be called, so we need an aRethrowExceptions argument.
            args.append(Argument("ExceptionHandling", "aExceptionHandling",
                                 "ReportExceptions"))
            return args
        # We want to allow the caller to pass in a "this" object, as
        # well as a JSContext.
        return [Argument("SafeJSContext", "cx"),
                Argument("HandleValue", "aThisObj")] + args

    def getCallSetup(self):
        if self.needThisHandling:
            # It's been done for us already
            return ""
        return (
            "CallSetup s(CallbackPreserveColor(), aRv, aExceptionHandling);\n"
            "JSContext* cx = *s.get_context();\n"
            "if (!cx) {\n"
            "    return Err(JSFailed);\n"
            "}\n")

    def getArgcDecl(self, immutable):
        if immutable:
            return CGGeneric(f"let argc = {self.argCountStr};")
        return CGGeneric(f"let mut argc = {self.argCountStr};")

    @staticmethod
    def ensureASCIIName(idlObject):
        type = "attribute" if idlObject.isAttr() else "operation"
        if re.match("[^\x20-\x7E]", idlObject.identifier.name):
            raise SyntaxError(f'Callback {type} name "{idlObject.identifier.name}" contains non-ASCII '
                              f"characters.  We can't handle that.  {idlObject.location}")
        if re.match('"', idlObject.identifier.name):
            raise SyntaxError(f"Callback {type} name '{idlObject.identifier.name}' contains "
                              "double-quote character.  We can't handle "
                              f"that.  {idlObject.location}")


class CallbackMethod(CallbackMember):
    def __init__(self, sig, name, descriptorProvider, needThisHandling):
        CallbackMember.__init__(self, sig, name, descriptorProvider,
                                needThisHandling)

    def getRvalDecl(self):
        if self.usingOutparam:
            return ""
        else:
            return "rooted!(in(*cx) let mut rval = UndefinedValue());\n"

    def getCall(self):
        if self.argCount > 0:
            argv = "argv.as_ptr() as *const JSVal"
            argc = "argc"
        else:
            argv = "ptr::null_mut()"
            argc = "0"
        suffix = "" if self.usingOutparam else ".handle_mut()"
        return (f"{self.getCallableDecl()}"
                f"rooted!(in(*cx) let rootedThis = {self.getThisObj()});\n"
                f"let ok = {self.getCallGuard()}Call(\n"
                "    *cx, rootedThis.handle(), callable.handle(),\n"
                "    &HandleValueArray {\n"
                f"        length_: {argc} as ::libc::size_t,\n"
                f"        elements_: {argv}\n"
                f"    }}, rval{suffix});\n"
                "maybe_resume_unwind();\n"
                "if !ok {\n"
                "    return Err(JSFailed);\n"
                "}\n")


class CallCallback(CallbackMethod):
    def __init__(self, callback, descriptorProvider):
        self.callback = callback
        CallbackMethod.__init__(self, callback.signatures()[0], "Call",
                                descriptorProvider, needThisHandling=True)

    def getThisObj(self):
        return "aThisObj.get()"

    def getCallableDecl(self):
        return "rooted!(in(*cx) let callable = ObjectValue(self.callback()));\n"

    def getCallGuard(self):
        if self.callback._treatNonObjectAsNull:
            return "!IsCallable(self.callback()) || "
        return ""


class CallbackOperationBase(CallbackMethod):
    """
    Common class for implementing various callback operations.
    """
    def __init__(self, signature, jsName, nativeName, descriptor, singleOperation):
        self.singleOperation = singleOperation
        self.methodName = jsName
        CallbackMethod.__init__(self, signature, nativeName, descriptor, singleOperation)

    def getThisObj(self):
        if not self.singleOperation:
            return "ObjectValue(self.callback())"
        # This relies on getCallableDecl declaring a boolean
        # isCallable in the case when we're a single-operation
        # interface.
        return "if isCallable { aThisObj.get() } else { ObjectValue(self.callback()) }"

    def getCallableDecl(self):
        getCallableFromProp = f'self.parent.get_callable_property(cx, "{self.methodName}")?'
        if not self.singleOperation:
            return f'rooted!(in(*cx) let callable =\n{getCallableFromProp});\n'
        callable = CGIndenter(
            CGIfElseWrapper('isCallable', CGGeneric('ObjectValue(self.callback())'), CGGeneric(getCallableFromProp))
        ).define()
        return ('let isCallable = IsCallable(self.callback());\n'
                'rooted!(in(*cx) let callable =\n'
                f"{callable});\n")

    def getCallGuard(self):
        return ""


class CallbackOperation(CallbackOperationBase):
    """
    Codegen actual WebIDL operations on callback interfaces.
    """
    def __init__(self, method, signature, descriptor):
        self.ensureASCIIName(method)
        jsName = method.identifier.name
        CallbackOperationBase.__init__(self, signature,
                                       jsName,
                                       MakeNativeName(descriptor.binaryNameFor(jsName, False)),
                                       descriptor, descriptor.interface.isSingleOperationInterface())
