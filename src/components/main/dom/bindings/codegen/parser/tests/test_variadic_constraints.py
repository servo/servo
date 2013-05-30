def WebIDLTest(parser, harness):
    threw = False
    try:
        results = parser.parse("""
            interface VariadicConstraints1 {
              void foo(byte... arg1, byte arg2);
            };
        """)

    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        results = parser.parse("""
            interface VariadicConstraints2 {
              void foo(byte... arg1, optional byte arg2);
            };
        """)

    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        results = parser.parse("""
            interface VariadicConstraints3 {
              void foo(optional byte... arg1);
            };
        """)

    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
