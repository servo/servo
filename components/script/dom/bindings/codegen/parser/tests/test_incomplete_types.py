import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        interface TestIncompleteTypes {
          attribute FooInterface attr1;

          FooInterface method1(FooInterface arg);
        };

        interface FooInterface {
        };
    """)

    results = parser.finish()

    harness.ok(True, "TestIncompleteTypes interface parsed without error.")
    harness.check(len(results), 2, "Should be two productions.")
    iface = results[0]
    harness.ok(isinstance(iface, WebIDL.IDLInterface),
               "Should be an IDLInterface")
    harness.check(iface.identifier.QName(), "::TestIncompleteTypes", "Interface has the right QName")
    harness.check(iface.identifier.name, "TestIncompleteTypes", "Interface has the right name")
    harness.check(len(iface.members), 2, "Expect 2 members")

    attr = iface.members[0]
    harness.ok(isinstance(attr, WebIDL.IDLAttribute),
               "Should be an IDLAttribute")
    method = iface.members[1]
    harness.ok(isinstance(method, WebIDL.IDLMethod),
               "Should be an IDLMethod")

    harness.check(attr.identifier.QName(), "::TestIncompleteTypes::attr1",
                  "Attribute has the right QName")
    harness.check(attr.type.name, "FooInterface",
                  "Previously unresolved type has the right name")

    harness.check(method.identifier.QName(), "::TestIncompleteTypes::method1",
                  "Attribute has the right QName")
    (returnType, args) = method.signatures()[0]
    harness.check(returnType.name, "FooInterface",
                  "Previously unresolved type has the right name")
    harness.check(args[0].type.name, "FooInterface",
                  "Previously unresolved type has the right name")
