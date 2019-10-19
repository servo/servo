def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            [NoInterfaceObject]
            interface TestConstructorNoInterfaceObject {
              constructor();
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()

    parser.parse("""
        [NoInterfaceObject, NamedConstructor=FooBar]
        interface TestNamedConstructorNoInterfaceObject {
        };
    """)

    # Test HTMLConstructor and NoInterfaceObject
    parser = parser.reset()

    threw = False
    try:
        parser.parse("""
            [NoInterfaceObject]
            interface TestHTMLConstructorNoInterfaceObject {
              [HTMLConstructor] constructor();
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
