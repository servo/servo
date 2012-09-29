import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        interface ForwardDeclared;
        interface ForwardDeclared;

        interface TestForwardDecl {
          attribute ForwardDeclared foo;
        };
    """)

    results = parser.finish()

    harness.ok(True, "TestForwardDeclared interface parsed without error.")
