import traceback

def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            [Global, Exposed=TestConstructorGlobal]
            interface TestConstructorGlobal {
              constructor();
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            [Global, Exposed=TestNamedConstructorGlobal,
             NamedConstructor=FooBar]
            interface TestNamedConstructorGlobal {
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            [NamedConstructor=FooBar, Global,
             Exposed=TestNamedConstructorGlobal]
            interface TestNamedConstructorGlobal {
            };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            [Global, HTMLConstructor, Exposed=TestHTMLConstructorGlobal]
            interface TestHTMLConstructorGlobal {
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            [HTMLConstructor, Global, Exposed=TestHTMLConstructorGlobal]
            interface TestHTMLConstructorGlobal {
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
