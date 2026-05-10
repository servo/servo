import WebIDL


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

        parser.finish()
    except WebIDL.WebIDLError:
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

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")
