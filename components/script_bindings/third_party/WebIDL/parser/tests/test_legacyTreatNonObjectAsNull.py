def WebIDLTest(parser, harness):
    parser.parse(
        """
        [LegacyTreatNonObjectAsNull] callback Function = any(any... arguments);
        """
    )

    results = parser.finish()

    callback = results[0]
    harness.check(callback._treatNonObjectAsNull, True, "Got the expected value")
