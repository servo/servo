import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        [NoInterfaceObject]
        interface TestExtendedAttr {
          [Unforgeable] readonly attribute byte b;
        };
    """)

    results = parser.finish()

    parser = parser.reset()
    parser.parse("""
        [Pref="foo.bar",Pref=flop]
        interface TestExtendedAttr {
          [Pref="foo.bar"] attribute byte b;
        };
    """)

    results = parser.finish()

    parser = parser.reset()
    parser.parse("""
        interface TestLenientThis {
          [LenientThis] attribute byte b;
        };
    """)

    results = parser.finish()
    harness.ok(results[0].members[0].hasLenientThis(),
               "Should have a lenient this")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface TestLenientThis2 {
              [LenientThis=something] attribute byte b;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "[LenientThis] must take no arguments")

    parser = parser.reset()
    parser.parse("""
        interface TestClamp {
          void testClamp([Clamp] long foo);
          void testNotClamp(long foo);
        };
    """)

    results = parser.finish()
    # Pull out the first argument out of the arglist of the first (and
    # only) signature.
    harness.ok(results[0].members[0].signatures()[0][1][0].clamp,
               "Should be clamped")
    harness.ok(not results[0].members[1].signatures()[0][1][0].clamp,
               "Should not be clamped")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface TestClamp2 {
              void testClamp([Clamp=something] long foo);
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "[Clamp] must take no arguments")

    parser = parser.reset()
    parser.parse("""
        interface TestEnforceRange {
          void testEnforceRange([EnforceRange] long foo);
          void testNotEnforceRange(long foo);
        };
    """)

    results = parser.finish()
    # Pull out the first argument out of the arglist of the first (and
    # only) signature.
    harness.ok(results[0].members[0].signatures()[0][1][0].enforceRange,
               "Should be enforceRange")
    harness.ok(not results[0].members[1].signatures()[0][1][0].enforceRange,
               "Should not be enforceRange")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface TestEnforceRange2 {
              void testEnforceRange([EnforceRange=something] long foo);
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "[EnforceRange] must take no arguments")
    
