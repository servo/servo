def WebIDLTest(parser, harness):

    parser.parse(
        """
        interface Foo {};
        [LegacyUnenumerableNamedProperties]
        interface Bar : Foo {
          getter long(DOMString name);
        };
        interface Baz : Bar {
          getter long(DOMString name);
        };
        """);
    results = parser.finish();
    harness.check(len(results), 3, "Should have three interfaces")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            [LegacyUnenumerableNamedProperties]
            interface NoNamedGetter {
            };
        """)

        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            [LegacyUnenumerableNamedProperties=Foo]
            interface ShouldNotHaveArg {
              getter long(DOMString name);
            };
        """)

        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            [LegacyUnenumerableNamedProperties]
            interface Foo {
              getter long(DOMString name);
            };
            interface Bar : Foo {};
            [LegacyUnenumerableNamedProperties]
            interface Baz : Bar {
              getter long(DOMString name);
            };
        """)

        results = parser.finish()
    except Exception, x:
        threw = True
    harness.ok(threw, "Should have thrown.")
