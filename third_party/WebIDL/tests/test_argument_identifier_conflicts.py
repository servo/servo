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

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
