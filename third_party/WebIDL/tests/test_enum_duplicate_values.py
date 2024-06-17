import WebIDL


def WebIDLTest(parser, harness):
    try:
        parser.parse(
            """
            enum TestEnumDuplicateValue {
              "",
              ""
            };
        """
        )
        harness.ok(False, "Should have thrown!")
    except WebIDL.WebIDLError:
        harness.ok(True, "Enum TestEnumDuplicateValue should throw")
