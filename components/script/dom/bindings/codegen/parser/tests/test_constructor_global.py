import traceback


def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse(
            """
            [Global, Exposed=TestConstructorGlobal]
            interface TestConstructorGlobal {
              constructor();
            };
        """
        )

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [Global, Exposed=TestLegacyFactoryFunctionGlobal,
             LegacyFactoryFunction=FooBar]
            interface TestLegacyFactoryFunctionGlobal {
            };
        """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [LegacyFactoryFunction=FooBar, Global,
             Exposed=TestLegacyFactoryFunctionGlobal]
            interface TestLegacyFactoryFunctionGlobal {
            };
        """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [Global, Exposed=TestHTMLConstructorGlobal]
            interface TestHTMLConstructorGlobal {
              [HTMLConstructor] constructor();
            };
        """
        )

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
