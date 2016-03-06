def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface _Promise {};
            interface A {
              legacycaller Promise<any> foo();
            };
        """)
        results = parser.finish()

    except:
        threw = True
    harness.ok(threw,
               "Should not allow Promise return values for legacycaller.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface _Promise {};
            interface A {
              Promise<any> foo();
              long foo(long arg);
            };
        """)
        results = parser.finish();
    except:
        threw = True
    harness.ok(threw,
               "Should not allow overloads which have both Promise and "
               "non-Promise return types.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface _Promise {};
            interface A {
              long foo(long arg);
              Promise<any> foo();
            };
        """)
        results = parser.finish();
    except:
        threw = True
    harness.ok(threw,
               "Should not allow overloads which have both Promise and "
               "non-Promise return types.")

    parser = parser.reset()
    parser.parse("""
        interface _Promise {};
        interface A {
          Promise<any> foo();
          Promise<any> foo(long arg);
        };
    """)
    results = parser.finish();

    harness.ok(True,
               "Should allow overloads which only have Promise and return "
               "types.")
