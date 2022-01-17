def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface UndefinedArgument1 {
              undefined foo(undefined arg2);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
