def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface VariadicConstraints1 {
              void foo(byte... arg1, byte arg2);
            };
        """)
        results = parser.finish()

    except:
        threw = True

    harness.ok(threw,
               "Should have thrown on variadic argument followed by required "
               "argument.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface VariadicConstraints2 {
              void foo(byte... arg1, optional byte arg2);
            };
        """)
        results = parser.finish();
    except:
        threw = True

    harness.ok(threw,
               "Should have thrown on variadic argument followed by optional "
               "argument.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface VariadicConstraints3 {
              void foo(optional byte... arg1);
            };
        """)
        results = parser.finish()

    except:
        threw = True

    harness.ok(threw,
               "Should have thrown on variadic argument explicitly flagged as "
               "optional.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface VariadicConstraints4 {
              void foo(byte... arg1 = 0);
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown on variadic argument with default value.")
