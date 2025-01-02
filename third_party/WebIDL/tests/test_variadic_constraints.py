import WebIDL


def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse(
            """
            interface VariadicConstraints1 {
              undefined foo(byte... arg1, byte arg2);
            };
        """
        )
        parser.finish()

    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should have thrown on variadic argument followed by required " "argument.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface VariadicConstraints2 {
              undefined foo(byte... arg1, optional byte arg2);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should have thrown on variadic argument followed by optional " "argument.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface VariadicConstraints3 {
              undefined foo(optional byte... arg1);
            };
        """
        )
        parser.finish()

    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should have thrown on variadic argument explicitly flagged as " "optional.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface VariadicConstraints4 {
              undefined foo(byte... arg1 = 0);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown on variadic argument with default value.")
