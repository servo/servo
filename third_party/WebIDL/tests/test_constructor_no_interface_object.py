def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse(
            """
            [LegacyNoInterfaceObject]
            interface TestConstructorLegacyNoInterfaceObject {
              constructor();
            };
        """
        )

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()

    parser.parse(
        """
        [LegacyNoInterfaceObject, LegacyFactoryFunction=FooBar]
        interface TestLegacyFactoryFunctionLegacyNoInterfaceObject {
        };
    """
    )

    # Test HTMLConstructor and LegacyNoInterfaceObject
    parser = parser.reset()

    threw = False
    try:
        parser.parse(
            """
            [LegacyNoInterfaceObject]
            interface TestHTMLConstructorLegacyNoInterfaceObject {
              [HTMLConstructor] constructor();
            };
        """
        )

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
