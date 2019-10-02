def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            [Global]
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
            [Global, NamedConstructor=FooBar]
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
            [NamedConstructor=FooBar, Global]
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
            [Global, HTMLConstructor]
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
            [HTMLConstructor, Global]
            interface TestHTMLConstructorGlobal {
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
