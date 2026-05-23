import WebIDL


def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse(
            """
            [Global=TestConstructorGlobal, Exposed=TestConstructorGlobal]
            interface TestConstructorGlobal {
              constructor();
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [Global=TestLegacyFactoryFunctionGlobal, Exposed=TestLegacyFactoryFunctionGlobal,
             LegacyFactoryFunction=FooBar]
            interface TestLegacyFactoryFunctionGlobal {
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [LegacyFactoryFunction=FooBar, Global=TestLegacyFactoryFunctionGlobal,
             Exposed=TestLegacyFactoryFunctionGlobal]
            interface TestLegacyFactoryFunctionGlobal {
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [Global=TestHTMLConstructorGlobal, Exposed=TestHTMLConstructorGlobal]
            interface TestHTMLConstructorGlobal {
              [HTMLConstructor] constructor();
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")
