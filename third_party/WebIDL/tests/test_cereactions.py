import WebIDL


def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse(
            """
            interface Foo {
              [CEReactions(DOMString a)] undefined foo(boolean arg2);
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for [CEReactions] with an argument")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Foo {
              [CEReactions(DOMString b)] readonly attribute boolean bar;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for [CEReactions] with an argument")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Foo {
              [CEReactions] attribute boolean bar;
            };
        """
        )

        parser.finish()
    except Exception as e:
        harness.ok(
            False,
            "Shouldn't have thrown for [CEReactions] used on writable attribute. %s"
            % e,
        )
        threw = True

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Foo {
              [CEReactions] undefined foo(boolean arg2);
            };
        """
        )

        parser.finish()
    except Exception as e:
        harness.ok(
            False,
            "Shouldn't have thrown for [CEReactions] used on regular operations. %s"
            % e,
        )
        threw = True

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Foo {
              [CEReactions] readonly attribute boolean A;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw, "Should have thrown for [CEReactions] used on a readonly attribute"
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            [CEReactions]
            interface Foo {
            }
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for [CEReactions] used on a interface")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          interface Foo {
            [CEReactions] getter any(DOMString name);
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for [CEReactions] used on a named getter")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          interface Foo {
            [CEReactions] legacycaller double compute(double x);
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for [CEReactions] used on a legacycaller")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          interface Foo {
            [CEReactions] stringifier DOMString ();
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for [CEReactions] used on a stringifier")
