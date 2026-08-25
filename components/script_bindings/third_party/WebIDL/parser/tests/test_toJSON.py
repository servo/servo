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
            f"interface Test {{ {type} toJSON(); }};",
            False,
            f"{type} should be a JSON type",
        )

        doTest(
            f"interface Test {{ sequence<{type}> toJSON(); }};",
            False,
            f"sequence<{type}> should be a JSON type",
        )

        doTest(
            f"dictionary Foo {{ {type} foo; }}; interface Test {{ Foo toJSON(); }}; ",
            False,
            f"dictionary containing only JSON type ({type}) should be a JSON type",
        )

        doTest(
            f"dictionary Foo {{ {type} foo; }}; dictionary Bar : Foo {{ }}; "
            "interface Test { Bar toJSON(); }; ",
            False,
            "dictionary whose ancestors only contain JSON types should be a JSON type",
        )

        doTest(
            f"dictionary Foo {{ any foo; }}; dictionary Bar : Foo {{ {type} bar; }};"
            "interface Test { Bar toJSON(); };",
            True,
            "dictionary whose ancestors contain non-JSON types should not be a JSON type",
        )

        doTest(
            f"interface Test {{ record<DOMString, {type}> toJSON(); }};",
            False,
            f"record<DOMString, {type}> should be a JSON type",
        )

        doTest(
            f"interface Test {{ record<ByteString, {type}> toJSON(); }};",
            False,
            f"record<ByteString, {type}> should be a JSON type",
        )

        doTest(
            f"interface Test {{ record<UTF8String, {type}> toJSON(); }};",
            False,
            f"record<UTF8String, {type}> should be a JSON type",
        )

        doTest(
            f"interface Test {{ record<USVString, {type}> toJSON(); }};",
            False,
            f"record<USVString, {type}> should be a JSON type",
        )

        otherUnionType = "Foo" if type != "object" else "long"
        doTest(
            "interface Foo { object toJSON(); };"
            f"interface Test {{ ({otherUnionType} or {type}) toJSON(); }};",
            False,
            f"union containing only JSON types ({otherUnionType} or {type}) should be a JSON type",
        )

        doTest(
            f"interface test {{ {type}? toJSON(); }};",
            False,
            f"Nullable type ({type}) should be a JSON type",
        )

        doTest(
            f"interface Foo : InterfaceWithoutToJSON {{ {type} toJSON(); }};"
            "interface Test { Foo toJSON(); };",
            False,
            "interface with toJSON should be a JSON type",
        )

    doTest(
        "interface Foo : InterfaceWithToJSON { };interface Test { Foo toJSON(); };",
        False,
        "inherited interface with toJSON should be a JSON type",
    )

    for type in nonJsonTypes:
        doTest(
            f"interface Test {{ {type} toJSON(); }};",
            True,
            f"{type} should not be a JSON type",
        )

        doTest(
            f"interface Test {{ sequence<{type}> toJSON(); }};",
            True,
            f"sequence<{type}> should not be a JSON type",
        )

        doTest(
            f"dictionary Foo {{ {type} foo; }}; interface Test {{ Foo toJSON(); }}; ",
            True,
            f"Dictionary containing a non-JSON type ({type}) should not be a JSON type",
        )

        doTest(
            f"dictionary Foo {{ {type} foo; }}; dictionary Bar : Foo {{ }}; "
            "interface Test { Bar toJSON(); }; ",
            True,
            "dictionary whose ancestors only contain non-JSON types should not be a JSON type",
        )

        doTest(
            f"interface Test {{ record<DOMString, {type}> toJSON(); }};",
            True,
            f"record<DOMString, {type}> should not be a JSON type",
        )

        doTest(
            f"interface Test {{ record<ByteString, {type}> toJSON(); }};",
            True,
            f"record<ByteString, {type}> should not be a JSON type",
        )

        doTest(
            f"interface Test {{ record<USVString, {type}> toJSON(); }};",
            True,
            f"record<USVString, {type}> should not be a JSON type",
        )

        if type != "any":
            doTest(
                "interface Foo { object toJSON(); }; "
                f"interface Test {{ (Foo or {type}) toJSON(); }};",
                True,
                f"union containing a non-JSON type ({type}) should not be a JSON type",
            )

            doTest(
                f"interface test {{ {type}? toJSON(); }};",
                True,
                f"Nullable type ({type}) should not be a JSON type",
            )

    doTest(
        "dictionary Foo { long foo; any bar; };interface Test { Foo toJSON(); };",
        True,
        "dictionary containing a non-JSON type should not be a JSON type",
    )

    doTest(
        "interface Foo : InterfaceWithoutToJSON { }; interface Test { Foo toJSON(); };",
        True,
        "interface without toJSON should not be a JSON type",
    )
