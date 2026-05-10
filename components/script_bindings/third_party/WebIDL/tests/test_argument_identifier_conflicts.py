import WebIDL


def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse(
            """
            interface ArgumentIdentifierConflict {
              undefined foo(boolean arg1, boolean arg1);
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")
