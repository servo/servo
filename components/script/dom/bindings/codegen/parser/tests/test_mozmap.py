import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        dictionary Dict {};
        interface MozMapArg {
          void foo(MozMap<Dict> arg);
        };
    """)

    results = parser.finish()

    harness.check(len(results), 2, "Should know about two things");
    harness.ok(isinstance(results[1], WebIDL.IDLInterface),
               "Should have an interface here");
    members = results[1].members
    harness.check(len(members), 1, "Should have one member")
    harness.ok(members[0].isMethod(), "Should have method")
    signature = members[0].signatures()[0]
    args = signature[1]
    harness.check(len(args), 1, "Should have one arg")
    harness.ok(args[0].type.isMozMap(), "Should have a MozMap type here")
    harness.ok(args[0].type.inner.isDictionary(),
               "Should have a dictionary inner type")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface MozMapVoidArg {
              void foo(MozMap<void> arg);
            };
        """)

        results = parser.finish()
    except Exception,x:
        threw = True

    harness.ok(threw, "Should have thrown.")
