import WebIDL

def WebIDLTest(parser, harness):
    try:
        parser.parse("""
            enum TestEmptyEnum {
            };
        """)

        harness.ok(False, "Should have thrown!")
    except:
        harness.ok(True, "Parsing TestEmptyEnum enum should fail")

    results = parser.finish()
