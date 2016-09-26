import WebIDL

def WebIDLTest(parser, harness):
    testData = [("::TestAttr%s::b", "b", "Byte%s", False),
                ("::TestAttr%s::rb", "rb", "Byte%s", True),
                ("::TestAttr%s::o", "o", "Octet%s", False),
                ("::TestAttr%s::ro", "ro", "Octet%s", True),
                ("::TestAttr%s::s", "s", "Short%s", False),
                ("::TestAttr%s::rs", "rs", "Short%s", True),
                ("::TestAttr%s::us", "us", "UnsignedShort%s", False),
                ("::TestAttr%s::rus", "rus", "UnsignedShort%s", True),
                ("::TestAttr%s::l", "l", "Long%s", False),
                ("::TestAttr%s::rl", "rl", "Long%s", True),
                ("::TestAttr%s::ul", "ul", "UnsignedLong%s", False),
                ("::TestAttr%s::rul", "rul", "UnsignedLong%s", True),
                ("::TestAttr%s::ll", "ll", "LongLong%s", False),
                ("::TestAttr%s::rll", "rll", "LongLong%s", True),
                ("::TestAttr%s::ull", "ull", "UnsignedLongLong%s", False),
                ("::TestAttr%s::rull", "rull", "UnsignedLongLong%s", True),
                ("::TestAttr%s::str", "str", "String%s", False),
                ("::TestAttr%s::rstr", "rstr", "String%s", True),
                ("::TestAttr%s::obj", "obj", "Object%s", False),
                ("::TestAttr%s::robj", "robj", "Object%s", True),
                ("::TestAttr%s::object", "object", "Object%s", False),
                ("::TestAttr%s::f", "f", "Float%s", False),
                ("::TestAttr%s::rf", "rf", "Float%s", True)]

    parser.parse("""
        interface TestAttr {
          attribute byte b;
          readonly attribute byte rb;
          attribute octet o;
          readonly attribute octet ro;
          attribute short s;
          readonly attribute short rs;
          attribute unsigned short us;
          readonly attribute unsigned short rus;
          attribute long l;
          readonly attribute long rl;
          attribute unsigned long ul;
          readonly attribute unsigned long rul;
          attribute long long ll;
          readonly attribute long long rll;
          attribute unsigned long long ull;
          readonly attribute unsigned long long rull;
          attribute DOMString str;
          readonly attribute DOMString rstr;
          attribute object obj;
          readonly attribute object robj;
          attribute object _object;
          attribute float f;
          readonly attribute float rf;
        };

        interface TestAttrNullable {
          attribute byte? b;
          readonly attribute byte? rb;
          attribute octet? o;
          readonly attribute octet? ro;
          attribute short? s;
          readonly attribute short? rs;
          attribute unsigned short? us;
          readonly attribute unsigned short? rus;
          attribute long? l;
          readonly attribute long? rl;
          attribute unsigned long? ul;
          readonly attribute unsigned long? rul;
          attribute long long? ll;
          readonly attribute long long? rll;
          attribute unsigned long long? ull;
          readonly attribute unsigned long long? rull;
          attribute DOMString? str;
          readonly attribute DOMString? rstr;
          attribute object? obj;
          readonly attribute object? robj;
          attribute object? _object;
          attribute float? f;
          readonly attribute float? rf;
        };
    """)

    results = parser.finish()

    def checkAttr(attr, QName, name, type, readonly):
        harness.ok(isinstance(attr, WebIDL.IDLAttribute),
                  "Should be an IDLAttribute")
        harness.ok(attr.isAttr(), "Attr is an Attr")
        harness.ok(not attr.isMethod(), "Attr is not an method")
        harness.ok(not attr.isConst(), "Attr is not a const")
        harness.check(attr.identifier.QName(), QName, "Attr has the right QName")
        harness.check(attr.identifier.name, name, "Attr has the right name")
        harness.check(str(attr.type), type, "Attr has the right type")
        harness.check(attr.readonly, readonly, "Attr's readonly state is correct")

    harness.ok(True, "TestAttr interface parsed without error.")
    harness.check(len(results), 2, "Should be two productions.")
    iface = results[0]
    harness.ok(isinstance(iface, WebIDL.IDLInterface),
               "Should be an IDLInterface")
    harness.check(iface.identifier.QName(), "::TestAttr", "Interface has the right QName")
    harness.check(iface.identifier.name, "TestAttr", "Interface has the right name")
    harness.check(len(iface.members), len(testData), "Expect %s members" % len(testData))

    attrs = iface.members

    for i in range(len(attrs)):
        data = testData[i]
        attr = attrs[i]
        (QName, name, type, readonly) = data
        checkAttr(attr, QName % "", name, type % "", readonly)

    iface = results[1]
    harness.ok(isinstance(iface, WebIDL.IDLInterface),
               "Should be an IDLInterface")
    harness.check(iface.identifier.QName(), "::TestAttrNullable", "Interface has the right QName")
    harness.check(iface.identifier.name, "TestAttrNullable", "Interface has the right name")
    harness.check(len(iface.members), len(testData), "Expect %s members" % len(testData))

    attrs = iface.members

    for i in range(len(attrs)):
        data = testData[i]
        attr = attrs[i]
        (QName, name, type, readonly) = data
        checkAttr(attr, QName % "Nullable", name, type % "OrNull", readonly)

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          interface A {
            [SetterThrows] readonly attribute boolean foo;
          };
        """)
        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should not allow [SetterThrows] on readonly attributes")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          interface A {
            [Throw] readonly attribute boolean foo;
          };
        """)
        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should spell [Throws] correctly")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          interface A {
            [SameObject] readonly attribute boolean foo;
          };
        """)
        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should not allow [SameObject] on attributes not of interface type")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          interface A {
            [SameObject] readonly attribute A foo;
          };
        """)
        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(not threw, "Should allow [SameObject] on attributes of interface type")
