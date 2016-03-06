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
