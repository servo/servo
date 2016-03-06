import WebIDL

def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface X {
              const sequence<long> foo = [];
            };
        """)

        results = parser.finish()
    except Exception,x:
        threw = True

    harness.ok(threw, "Constant cannot have [] as a default value")

    parser = parser.reset()

    parser.parse("""
        interface X {
          void foo(optional sequence<long> arg = []);
        };
    """)
    results = parser.finish();

    harness.ok(isinstance(
        results[0].members[0].signatures()[0][1][0].defaultValue,
        WebIDL.IDLEmptySequenceValue),
               "Should have IDLEmptySequenceValue as default value of argument")

    parser = parser.reset()

    parser.parse("""
        dictionary X {
          sequence<long> foo = [];
        };
    """)
    results = parser.finish();

    harness.ok(isinstance(results[0].members[0].defaultValue,
                          WebIDL.IDLEmptySequenceValue),
               "Should have IDLEmptySequenceValue as default value of "
               "dictionary member")

