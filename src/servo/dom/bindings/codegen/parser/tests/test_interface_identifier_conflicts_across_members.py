def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface IdentifierConflictAcrossMembers1 {
              const byte thing1 = 1;
              readonly attribute long thing1;
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface IdentifierConflictAcrossMembers2 {
              readonly attribute long thing1;
              const byte thing1 = 1;
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface IdentifierConflictAcrossMembers3 {
              getter boolean thing1(DOMString name);
              readonly attribute long thing1;
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface IdentifierConflictAcrossMembers1 {
              const byte thing1 = 1;
              long thing1();
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
