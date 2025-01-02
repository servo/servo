import WebIDL


def WebIDLTest(parser, harness):
    parser.parse(
        """
        [LegacyNoInterfaceObject]
        interface TestExtendedAttr {
          [LegacyUnforgeable] readonly attribute byte b;
        };
    """
    )

    parser.finish()

    parser = parser.reset()
    parser.parse(
        """
        [Pref="foo.bar",Pref=flop]
        interface TestExtendedAttr {
          [Pref="foo.bar"] attribute byte b;
        };
    """
    )

    parser.finish()

    parser = parser.reset()
    parser.parse(
        """
        interface TestLegacyLenientThis {
          [LegacyLenientThis] attribute byte b;
        };
    """
    )

    results = parser.finish()
    harness.ok(
        results[0].members[0].hasLegacyLenientThis(), "Should have a lenient this"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface TestLegacyLenientThis2 {
              [LegacyLenientThis=something] attribute byte b;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "[LegacyLenientThis] must take no arguments")

    parser = parser.reset()
    parser.parse(
        """
        interface TestClamp {
          undefined testClamp([Clamp] long foo);
          undefined testNotClamp(long foo);
        };
    """
    )

    results = parser.finish()
    # Pull out the first argument out of the arglist of the first (and
    # only) signature.
    harness.ok(
        results[0].members[0].signatures()[0][1][0].type.hasClamp(), "Should be clamped"
    )
    harness.ok(
        not results[0].members[1].signatures()[0][1][0].type.hasClamp(),
        "Should not be clamped",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface TestClamp2 {
              undefined testClamp([Clamp=something] long foo);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "[Clamp] must take no arguments")

    parser = parser.reset()
    parser.parse(
        """
        interface TestEnforceRange {
          undefined testEnforceRange([EnforceRange] long foo);
          undefined testNotEnforceRange(long foo);
        };
    """
    )

    results = parser.finish()
    # Pull out the first argument out of the arglist of the first (and
    # only) signature.
    harness.ok(
        results[0].members[0].signatures()[0][1][0].type.hasEnforceRange(),
        "Should be enforceRange",
    )
    harness.ok(
        not results[0].members[1].signatures()[0][1][0].type.hasEnforceRange(),
        "Should not be enforceRange",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface TestEnforceRange2 {
              undefined testEnforceRange([EnforceRange=something] long foo);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "[EnforceRange] must take no arguments")
