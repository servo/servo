def WebIDLTest(parser, harness):
    parser.parse(
        """
        interface Foo {
        };

        interface Bar {
        };

        typedef (Foo or Bar) Baz;

        interface Test {
          attribute Baz length;
        };
    """
    )

    parser.finish()

    harness.ok(True, "TestUnionAsLengthAttr interface parsed without error.")
