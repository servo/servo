import WebIDL


def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse(
            """
            interface AttrSequenceType {
              attribute sequence<object> foo;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Attribute type must not be a sequence type")

    parser.reset()

    threw = False
    try:
        parser.parse(
            """
            interface AttrUnionWithSequenceType {
              attribute (sequence<object> or DOMString) foo;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Attribute type must not be a union with a sequence member type")

    parser.reset()

    threw = False
    try:
        parser.parse(
            """
            interface AttrNullableUnionWithSequenceType {
              attribute (sequence<object>? or DOMString) foo;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Attribute type must not be a union with a nullable sequence " "member type",
    )

    parser.reset()

    threw = False
    try:
        parser.parse(
            "\n"
            "    interface AttrUnionWithUnionWithSequenceType {\n"
            "      attribute ((sequence<object> or DOMString) or "
            "AttrUnionWithUnionWithSequenceType) foo;\n"
            "    };\n"
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Attribute type must not be a union type with a union member "
        "type that has a sequence member type",
    )
