def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            [Constructor, NoInterfaceObject]
            interface TestConstructorNoInterfaceObject {
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            [NoInterfaceObject, Constructor]
            interface TestConstructorNoInterfaceObject {
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
            [NoInterfaceObject, HTMLConstructor]
            interface TestHTMLConstructorNoInterfaceObject {
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
            [HTMLConstructor, NoInterfaceObject]
            interface TestHTMLConstructorNoInterfaceObject {
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
