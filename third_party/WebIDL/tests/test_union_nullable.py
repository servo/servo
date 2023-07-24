def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse(
            """
            interface OneNullableInUnion {
              undefined foo((object? or DOMString?) arg);
            };
        """
        )

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Two nullable member types of a union should have thrown.")

    parser.reset()
    threw = False

    try:
        parser.parse(
            """
            interface NullableInNullableUnion {
              undefined foo((object? or DOMString)? arg);
            };
        """
        )

        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "A nullable union type with a nullable member type should have " "thrown.",
    )

    parser.reset()
    threw = False

    try:
        parser.parse(
            """
            interface NullableInUnionNullableUnionHelper {
            };
            interface NullableInUnionNullableUnion {
              undefined foo(((object? or DOMString) or NullableInUnionNullableUnionHelper)? arg);
            };
        """
        )

        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "A nullable union type with a nullable member type should have " "thrown.",
    )
