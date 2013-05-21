def WebIDLTest(parser, harness):
    parser.parse("")
    parser.finish()
    harness.ok(True, "Parsing nothing doesn't throw.")
    parser.parse("interface Foo {};")
    parser.finish()
    harness.ok(True, "Parsing a silly interface doesn't throw.")
