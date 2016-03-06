def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface TestIface {
              static attribute boolean prototype;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "The identifier of a static attribute must not be 'prototype'")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface TestIface {
              static boolean prototype();
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "The identifier of a static operation must not be 'prototype'")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface TestIface {
              const boolean prototype = true;
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "The identifier of a constant must not be 'prototype'")

    # Make sure that we can parse non-static attributes with 'prototype' as identifier.
    parser = parser.reset()
    parser.parse("""
        interface TestIface {
          attribute boolean prototype;
        };
    """)
    results = parser.finish()

    testIface = results[0];
    harness.check(testIface.members[0].isStatic(), False, "Attribute should not be static")
    harness.check(testIface.members[0].identifier.name, "prototype", "Attribute identifier should be 'prototype'")

    # Make sure that we can parse non-static operations with 'prototype' as identifier.
    parser = parser.reset()
    parser.parse("""
        interface TestIface {
          boolean prototype();
        };
    """)
    results = parser.finish()

    testIface = results[0];
    harness.check(testIface.members[0].isStatic(), False, "Operation should not be static")
    harness.check(testIface.members[0].identifier.name, "prototype", "Operation identifier should be 'prototype'")

    # Make sure that we can parse dictionary members with 'prototype' as identifier.
    parser = parser.reset()
    parser.parse("""
        dictionary TestDict {
          boolean prototype;
        };
    """)
    results = parser.finish()

    testDict = results[0];
    harness.check(testDict.members[0].identifier.name, "prototype", "Dictionary member should be 'prototype'")

