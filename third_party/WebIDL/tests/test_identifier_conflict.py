def WebIDLTest(parser, harness):
    try:
        parser.parse(
            """
            enum Foo { "a" };
            interface Foo;
        """
        )
        parser.finish()
        harness.ok(False, "Should fail to parse")
    except Exception as e:
        harness.ok(
            "Name collision" in str(e), "Should have name collision for interface"
        )

    parser = parser.reset()
    try:
        parser.parse(
            """
            dictionary Foo { long x; };
            enum Foo { "a" };
        """
        )
        parser.finish()
        harness.ok(False, "Should fail to parse")
    except Exception as e:
        harness.ok(
            "Name collision" in str(e), "Should have name collision for dictionary"
        )

    parser = parser.reset()
    try:
        parser.parse(
            """
            enum Foo { "a" };
            enum Foo { "b" };
        """
        )
        parser.finish()
        harness.ok(False, "Should fail to parse")
    except Exception as e:
        harness.ok(
            "Multiple unresolvable definitions" in str(e),
            "Should have name collision for dictionary",
        )
