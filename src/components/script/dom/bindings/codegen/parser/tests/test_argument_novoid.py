def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface VoidArgument1 {
              void foo(void arg2);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
