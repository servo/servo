import WebIDL


def WebIDLTest(parser, harness):
    parser.parse(
        """
        [TreatNonCallableAsNull] callback Function = any(any... arguments);

        interface TestTreatNonCallableAsNull1 {
          attribute Function? onfoo;
          attribute Function onbar;
        };
    """
    )

    results = parser.finish()

    iface = results[1]
    attr = iface.members[0]
    harness.check(attr.type.treatNonCallableAsNull(), True, "Got the expected value")
    attr = iface.members[1]
    harness.check(attr.type.treatNonCallableAsNull(), False, "Got the expected value")

    parser = parser.reset()

    threw = False
    try:
        parser.parse(
            """
            callback Function = any(any... arguments);

            interface TestTreatNonCallableAsNull2 {
              [TreatNonCallableAsNull] attribute Function onfoo;
            };
        """
        )

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()

    threw = False
    try:
        parser.parse(
            """
            callback Function = any(any... arguments);

            [TreatNonCallableAsNull]
            interface TestTreatNonCallableAsNull3 {
               attribute Function onfoo;
            };
        """
        )

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()

    threw = False
    try:
        parser.parse(
            """
            [TreatNonCallableAsNull, LegacyTreatNonObjectAsNull]
            callback Function = any(any... arguments);
        """
        )

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
