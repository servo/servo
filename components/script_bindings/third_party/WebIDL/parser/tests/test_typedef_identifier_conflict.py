def WebIDLTest(parser, harness):
    exception = None
    try:
        parser.parse(
            """
            typedef long foo;
            typedef long foo;
            """
        )

        parser.finish()
    except Exception as e:
        exception = e

    harness.ok(exception, "Should have thrown.")
    harness.ok(
        "Multiple unresolvable definitions of identifier 'foo'" in str(exception),
        "Should have a sane exception message",
    )
