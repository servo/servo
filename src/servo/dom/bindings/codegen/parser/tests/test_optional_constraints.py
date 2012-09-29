def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface OptionalConstraints1 {
              void foo(optional byte arg1, byte arg2);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
