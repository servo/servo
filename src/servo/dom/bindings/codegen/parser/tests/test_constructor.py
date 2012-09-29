import WebIDL

def WebIDLTest(parser, harness):
    def checkArgument(argument, QName, name, type, optional, variadic):
        harness.ok(isinstance(argument, WebIDL.IDLArgument),
                   "Should be an IDLArgument")
        harness.check(argument.identifier.QName(), QName, "Argument has the right QName")
        harness.check(argument.identifier.name, name, "Argument has the right name")
        harness.check(str(argument.type), type, "Argument has the right return type")
        harness.check(argument.optional, optional, "Argument has the right optional value")
        harness.check(argument.variadic, variadic, "Argument has the right variadic value")

    def checkMethod(method, QName, name, signatures,
                    static=False, getter=False, setter=False, creator=False,
                    deleter=False, legacycaller=False, stringifier=False):
        harness.ok(isinstance(method, WebIDL.IDLMethod),
                   "Should be an IDLMethod")
        harness.ok(method.isMethod(), "Method is a method")
        harness.ok(not method.isAttr(), "Method is not an attr")
        harness.ok(not method.isConst(), "Method is not a const")
        harness.check(method.identifier.QName(), QName, "Method has the right QName")
        harness.check(method.identifier.name, name, "Method has the right name")
        harness.check(method.isStatic(), static, "Method has the correct static value")
        harness.check(method.isGetter(), getter, "Method has the correct getter value")
        harness.check(method.isSetter(), setter, "Method has the correct setter value")
        harness.check(method.isCreator(), creator, "Method has the correct creator value")
        harness.check(method.isDeleter(), deleter, "Method has the correct deleter value")
        harness.check(method.isLegacycaller(), legacycaller, "Method has the correct legacycaller value")
        harness.check(method.isStringifier(), stringifier, "Method has the correct stringifier value")
        harness.check(len(method.signatures()), len(signatures), "Method has the correct number of signatures")

        sigpairs = zip(method.signatures(), signatures)
        for (gotSignature, expectedSignature) in sigpairs:
            (gotRetType, gotArgs) = gotSignature
            (expectedRetType, expectedArgs) = expectedSignature

            harness.check(str(gotRetType), expectedRetType,
                          "Method has the expected return type.")

            for i in range(0, len(gotArgs)):
                (QName, name, type, optional, variadic) = expectedArgs[i]
                checkArgument(gotArgs[i], QName, name, type, optional, variadic)

    parser.parse("""
        [Constructor]
        interface TestConstructorNoArgs {
        };

        [Constructor(DOMString name)]
        interface TestConstructorWithArgs {
        };

        [Constructor(object foo), Constructor(boolean bar)]
        interface TestConstructorOverloads {
        };
    """)
    results = parser.finish()
    harness.check(len(results), 3, "Should be two productions")
    harness.ok(isinstance(results[0], WebIDL.IDLInterface),
               "Should be an IDLInterface")
    harness.ok(isinstance(results[1], WebIDL.IDLInterface),
               "Should be an IDLInterface")

    checkMethod(results[0].ctor(), "::TestConstructorNoArgs::constructor",
                "constructor", [("TestConstructorNoArgs (Wrapper)", [])])
    checkMethod(results[1].ctor(), "::TestConstructorWithArgs::constructor",
                "constructor",
                [("TestConstructorWithArgs (Wrapper)",
                 [("::TestConstructorWithArgs::constructor::name", "name", "String", False, False)])])
    checkMethod(results[2].ctor(), "::TestConstructorOverloads::constructor",
                "constructor",
                [("TestConstructorOverloads (Wrapper)",
                 [("::TestConstructorOverloads::constructor::foo", "foo", "Object", False, False)]),
                 ("TestConstructorOverloads (Wrapper)",
                 [("::TestConstructorOverloads::constructor::bar", "bar", "Boolean", False, False)])])
