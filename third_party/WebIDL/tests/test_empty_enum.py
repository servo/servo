import WebIDL


def WebIDLTest(parser, harness):
    try:
        parser.parse(
            """
            enum TestEmptyEnum {
            };
        """
        )

        harness.ok(False, "Should have thrown!")
    except WebIDL.WebIDLError:
        harness.ok(True, "Parsing TestEmptyEnum enum should fail")

    parser.finish()
