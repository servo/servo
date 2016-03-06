import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        interface TestStringifier {
          stringifier;
        };
    """)

    results = parser.finish()

    harness.ok(isinstance(results[0].members[0], WebIDL.IDLMethod),
               "Stringifer should be method")

    parser = parser.reset()

    threw = False
    try:
        parser.parse("""
            interface TestStringifier {
              stringifier;
              stringifier;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow two 'stringifier;'")

    parser = parser.reset()

    threw = False
    try:
        parser.parse("""
            interface TestStringifier {
              stringifier;
              stringifier DOMString foo();
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow a 'stringifier;' and a 'stringifier()'")

