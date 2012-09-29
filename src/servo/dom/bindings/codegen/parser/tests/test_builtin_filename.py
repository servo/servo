import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
        interface Test {
          attribute long b;
        };
    """);

    attr = parser.finish()[0].members[0]
    harness.check(attr.type.filename(), '<builtin>', 'Filename on builtin type')
