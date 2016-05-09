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

    parser = parser.reset()
    parser.parse("""
        callback interface TestCallbackInterface1 {
          void foo();
        };
        callback interface TestCallbackInterface2 {
          void foo(DOMString arg);
          void foo(TestCallbackInterface1 arg);
        };
        callback interface TestCallbackInterface3 {
          void foo(DOMString arg);
          void foo(TestCallbackInterface1 arg);
          static void bar();
        };
        callback interface TestCallbackInterface4 {
          void foo(DOMString arg);
          void foo(TestCallbackInterface1 arg);
          static void bar();
          const long baz = 5;
        };
        callback interface TestCallbackInterface5 {
          static attribute boolean bool;
          void foo();
        };
        callback interface TestCallbackInterface6 {
          void foo(DOMString arg);
          void foo(TestCallbackInterface1 arg);
          void bar();
        };
        callback interface TestCallbackInterface7 {
          static attribute boolean bool;
        };
        callback interface TestCallbackInterface8 {
          attribute boolean bool;
        };
        callback interface TestCallbackInterface9 : TestCallbackInterface1 {
          void foo();
        };
        callback interface TestCallbackInterface10 : TestCallbackInterface1 {
          void bar();
        };
    """)
    results = parser.finish()
    for (i, iface) in enumerate(results):
      harness.check(iface.isSingleOperationInterface(), i < 4,
                    "Interface %s should be a single operation interface" %
                    iface.identifier.name)
