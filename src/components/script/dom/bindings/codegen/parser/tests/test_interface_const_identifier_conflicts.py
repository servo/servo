def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface IdentifierConflict {
              const byte thing1 = 1;
              const unsigned long thing1 = 1;
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
