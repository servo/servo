import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        enum TestEnum {
          "",
          "foo",
          "bar"
        };

        interface TestEnumInterface {
          TestEnum doFoo(boolean arg);
          readonly attribute TestEnum foo;
        };
    """)

    results = parser.finish()

    harness.ok(True, "TestEnumInterfaces interface parsed without error.")
    harness.check(len(results), 2, "Should be one production")
    harness.ok(isinstance(results[0], WebIDL.IDLEnum),
               "Should be an IDLEnum")
    harness.ok(isinstance(results[1], WebIDL.IDLInterface),
               "Should be an IDLInterface")

    enum = results[0]
    harness.check(enum.identifier.QName(), "::TestEnum", "Enum has the right QName")
    harness.check(enum.identifier.name, "TestEnum", "Enum has the right name")
    harness.check(enum.values(), ["", "foo", "bar"], "Enum has the right values")

    iface = results[1]

    harness.check(iface.identifier.QName(), "::TestEnumInterface", "Interface has the right QName")
    harness.check(iface.identifier.name, "TestEnumInterface", "Interface has the right name")
    harness.check(iface.parent, None, "Interface has no parent")

    members = iface.members
    harness.check(len(members), 2, "Should be one production")
    harness.ok(isinstance(members[0], WebIDL.IDLMethod),
               "Should be an IDLMethod")
    method = members[0]
    harness.check(method.identifier.QName(), "::TestEnumInterface::doFoo",
                  "Method has correct QName")
    harness.check(method.identifier.name, "doFoo", "Method has correct name")

    signatures = method.signatures()
    harness.check(len(signatures), 1, "Expect one signature")

    (returnType, arguments) = signatures[0]
    harness.check(str(returnType), "TestEnum (Wrapper)", "Method type is the correct name")
    harness.check(len(arguments), 1, "Method has the right number of arguments")
    arg = arguments[0]
    harness.ok(isinstance(arg, WebIDL.IDLArgument), "Should be an IDLArgument")
    harness.check(str(arg.type), "Boolean", "Argument has the right type")

    attr = members[1]
    harness.check(attr.identifier.QName(), "::TestEnumInterface::foo",
                  "Attr has correct QName")
    harness.check(attr.identifier.name, "foo", "Attr has correct name")

    harness.check(str(attr.type), "TestEnum (Wrapper)", "Attr type is the correct name")

    # Now reset our parser
    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          enum Enum {
            "a",
            "b",
            "c"
          };
          interface TestInterface {
            void foo(optional Enum e = "d");
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow a bogus default value for an enum")
