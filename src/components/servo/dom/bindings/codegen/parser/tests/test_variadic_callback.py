import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        callback TestVariadicCallback = any(any... arguments);
    """)

    results = parser.finish()

    harness.ok(True, "TestVariadicCallback callback parsed without error.")
