import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        interface TestCallback {
          attribute CallbackType? listener;
        };

        callback CallbackType = boolean (unsigned long arg);
    """)

    results = parser.finish()

    harness.ok(True, "TestCallback interface parsed without error.")
    harness.check(len(results), 2, "Should be one production.")
    iface = results[0]
    harness.ok(isinstance(iface, WebIDL.IDLInterface),
               "Should be an IDLInterface")
    harness.check(iface.identifier.QName(), "::TestCallback", "Interface has the right QName")
    harness.check(iface.identifier.name, "TestCallback", "Interface has the right name")
    harness.check(len(iface.members), 1, "Expect %s members" % 1)

    attr = iface.members[0]
    harness.ok(isinstance(attr, WebIDL.IDLAttribute),
               "Should be an IDLAttribute")
    harness.ok(attr.isAttr(), "Should be an attribute")
    harness.ok(not attr.isMethod(), "Attr is not an method")
    harness.ok(not attr.isConst(), "Attr is not a const")
    harness.check(attr.identifier.QName(), "::TestCallback::listener", "Attr has the right QName")
    harness.check(attr.identifier.name, "listener", "Attr has the right name")
    t = attr.type
    harness.ok(not isinstance(t, WebIDL.IDLWrapperType), "Attr has the right type")
    harness.ok(isinstance(t, WebIDL.IDLNullableType), "Attr has the right type")
    harness.ok(t.isCallback(), "Attr has the right type")
