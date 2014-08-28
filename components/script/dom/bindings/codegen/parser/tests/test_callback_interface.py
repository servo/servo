import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        callback interface TestCallbackInterface {
          attribute boolean bool;
        };
    """)

    results = parser.finish()

    iface = results[0]

    harness.ok(iface.isCallback(), "Interface should be a callback")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface TestInterface {
            };
            callback interface TestCallbackInterface : TestInterface {
              attribute boolean bool;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow non-callback parent of callback interface")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface TestInterface : TestCallbackInterface {
            };
            callback interface TestCallbackInterface {
              attribute boolean bool;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should not allow callback parent of non-callback interface")

