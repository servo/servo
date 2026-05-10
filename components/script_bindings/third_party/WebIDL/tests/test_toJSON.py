import WebIDL


def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse(
            """
            interface Test {
              object toJSON();
            };
            """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(not threw, "Should allow a toJSON method.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Test {
              object toJSON(object arg);
              object toJSON(long arg);
            };
            """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should not allow overloads of a toJSON method.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Test {
              object toJSON(object arg);
            };
            """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should not allow a toJSON method with arguments.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Test {
              long toJSON();
            };
            """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(not threw, "Should allow a toJSON method with 'long' as return type.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Test {
              [Default] object toJSON();
            };
            """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        not threw, "Should allow a default toJSON method with 'object' as return type."
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Test {
              [Default] long toJSON();
            };
            """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should not allow a default toJSON method with non-'object' as return type.",
    )

    JsonTypes = [
        "byte",
        "octet",
        "short",
        "unsigned short",
        "long",
        "unsigned long",
        "long long",
        "unsigned long long",
        "float",
        "unrestricted float",
        "double",
        "unrestricted double",
        "boolean",
        "DOMString",
        "ByteString",
        "UTF8String",
        "USVString",
        "Enum",
        "InterfaceWithToJSON",
        "object",
    ]

    nonJsonTypes = [
        "InterfaceWithoutToJSON",
        "any",
        "Int8Array",
        "Int16Array",
        "Int32Array",
        "Uint8Array",
        "Uint16Array",
        "Uint32Array",
        "Uint8ClampedArray",
        "Float32Array",
        "Float64Array",
        "ArrayBuffer",
    ]

    def doTest(testIDL, shouldThrow, description):
        p = parser.reset()
        threw = False
        try:
            p.parse(
                testIDL
                + """
                enum Enum { "a", "b", "c" };
                interface InterfaceWithToJSON { long toJSON(); };
                interface InterfaceWithoutToJSON {};
                """
            )
            p.finish()
        except Exception as x:
            threw = True
            harness.ok(x.message == "toJSON method has non-JSON return type", x)
        harness.check(threw, shouldThrow, description)

    for type in JsonTypes:
        doTest(
            "interface Test { %s toJSON(); };" % type,
            False,
            "%s should be a JSON type" % type,
        )

        doTest(
            "interface Test { sequence<%s> toJSON(); };" % type,
            False,
            "sequence<%s> should be a JSON type" % type,
        )

        doTest(
            "dictionary Foo { %s foo; }; " "interface Test { Foo toJSON(); }; " % type,
            False,
            "dictionary containing only JSON type (%s) should be a JSON type" % type,
        )

        doTest(
            "dictionary Foo { %s foo; }; dictionary Bar : Foo { }; "
            "interface Test { Bar toJSON(); }; " % type,
            False,
            "dictionary whose ancestors only contain JSON types should be a JSON type",
        )

        doTest(
            "dictionary Foo { any foo; }; dictionary Bar : Foo { %s bar; };"
            "interface Test { Bar toJSON(); };" % type,
            True,
            "dictionary whose ancestors contain non-JSON types should not be a JSON type",
        )

        doTest(
            "interface Test { record<DOMString, %s> toJSON(); };" % type,
            False,
            "record<DOMString, %s> should be a JSON type" % type,
        )

        doTest(
            "interface Test { record<ByteString, %s> toJSON(); };" % type,
            False,
            "record<ByteString, %s> should be a JSON type" % type,
        )

        doTest(
            "interface Test { record<UTF8String, %s> toJSON(); };" % type,
            False,
            "record<UTF8String, %s> should be a JSON type" % type,
        )

        doTest(
            "interface Test { record<USVString, %s> toJSON(); };" % type,
            False,
            "record<USVString, %s> should be a JSON type" % type,
        )

        otherUnionType = "Foo" if type != "object" else "long"
        doTest(
            "interface Foo { object toJSON(); };"
            "interface Test { (%s or %s) toJSON(); };" % (otherUnionType, type),
            False,
            "union containing only JSON types (%s or %s) should be a JSON type"
            % (otherUnionType, type),
        )

        doTest(
            "interface test { %s? toJSON(); };" % type,
            False,
            "Nullable type (%s) should be a JSON type" % type,
        )

        doTest(
            "interface Foo : InterfaceWithoutToJSON { %s toJSON(); };"
            "interface Test { Foo toJSON(); };" % type,
            False,
            "interface with toJSON should be a JSON type",
        )

    doTest(
        "interface Foo : InterfaceWithToJSON { };" "interface Test { Foo toJSON(); };",
        False,
        "inherited interface with toJSON should be a JSON type",
    )

    for type in nonJsonTypes:
        doTest(
            "interface Test { %s toJSON(); };" % type,
            True,
            "%s should not be a JSON type" % type,
        )

        doTest(
            "interface Test { sequence<%s> toJSON(); };" % type,
            True,
            "sequence<%s> should not be a JSON type" % type,
        )

        doTest(
            "dictionary Foo { %s foo; }; " "interface Test { Foo toJSON(); }; " % type,
            True,
            "Dictionary containing a non-JSON type (%s) should not be a JSON type"
            % type,
        )

        doTest(
            "dictionary Foo { %s foo; }; dictionary Bar : Foo { }; "
            "interface Test { Bar toJSON(); }; " % type,
            True,
            "dictionary whose ancestors only contain non-JSON types should not be a JSON type",
        )

        doTest(
            "interface Test { record<DOMString, %s> toJSON(); };" % type,
            True,
            "record<DOMString, %s> should not be a JSON type" % type,
        )

        doTest(
            "interface Test { record<ByteString, %s> toJSON(); };" % type,
            True,
            "record<ByteString, %s> should not be a JSON type" % type,
        )

        doTest(
            "interface Test { record<USVString, %s> toJSON(); };" % type,
            True,
            "record<USVString, %s> should not be a JSON type" % type,
        )

        if type != "any":
            doTest(
                "interface Foo { object toJSON(); }; "
                "interface Test { (Foo or %s) toJSON(); };" % type,
                True,
                "union containing a non-JSON type (%s) should not be a JSON type"
                % type,
            )

            doTest(
                "interface test { %s? toJSON(); };" % type,
                True,
                "Nullable type (%s) should not be a JSON type" % type,
            )

    doTest(
        "dictionary Foo { long foo; any bar; };" "interface Test { Foo toJSON(); };",
        True,
        "dictionary containing a non-JSON type should not be a JSON type",
    )

    doTest(
        "interface Foo : InterfaceWithoutToJSON { }; "
        "interface Test { Foo toJSON(); };",
        True,
        "interface without toJSON should not be a JSON type",
    )
