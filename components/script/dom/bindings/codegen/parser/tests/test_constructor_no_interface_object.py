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
