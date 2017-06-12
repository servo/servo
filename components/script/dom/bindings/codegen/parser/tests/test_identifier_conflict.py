# Import the WebIDL module, so we can do isinstance checks and whatnot
import WebIDL

def WebIDLTest(parser, harness):
    try:
        parser.parse("""
            enum Foo { "a" };
            interface Foo;
        """)
        results = parser.finish()
        harness.ok(False, "Should fail to parse")
    except Exception, e:
        harness.ok("Name collision" in e.message,
                   "Should have name collision for interface")

    parser = parser.reset()
    try:
        parser.parse("""
            dictionary Foo { long x; };
            enum Foo { "a" };
        """)
        results = parser.finish()
        harness.ok(False, "Should fail to parse")
    except Exception, e:
        harness.ok("Name collision" in e.message,
                   "Should have name collision for dictionary")

    parser = parser.reset()
    try:
        parser.parse("""
            enum Foo { "a" };
            enum Foo { "b" };
        """)
        results = parser.finish()
        harness.ok(False, "Should fail to parse")
    except Exception, e:
        harness.ok("Multiple unresolvable definitions" in e.message,
                   "Should have name collision for dictionary")

