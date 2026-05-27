import WebIDL


def WebIDLTest(parser, harness):
    parser.parse(
        """
            interface Child : Parent {
            };
            interface Parent {
              [LegacyUnforgeable] readonly attribute long foo;
            };
        """
    )

    results = parser.finish()
    harness.check(
        len(results),
        2,
        "Should be able to inherit from an interface with "
        "[LegacyUnforgeable] properties.",
    )

    parser = parser.reset()
    parser.parse(
        """
            interface Child : Parent {
              const short foo = 10;
            };
            interface Parent {
              [LegacyUnforgeable] readonly attribute long foo;
            };
        """
    )

    results = parser.finish()
    harness.check(
        len(results),
        2,
        "Should be able to inherit from an interface with "
        "[LegacyUnforgeable] properties even if we have a constant with "
        "the same name.",
    )

    parser = parser.reset()
    parser.parse(
        """
            interface Child : Parent {
              static attribute short foo;
            };
            interface Parent {
              [LegacyUnforgeable] readonly attribute long foo;
            };
        """
    )

    results = parser.finish()
    harness.check(
        len(results),
        2,
        "Should be able to inherit from an interface with "
        "[LegacyUnforgeable] properties even if we have a static attribute "
        "with the same name.",
    )

    parser = parser.reset()
    parser.parse(
        """
            interface Child : Parent {
              static undefined foo();
            };
            interface Parent {
              [LegacyUnforgeable] readonly attribute long foo;
            };
        """
    )

    results = parser.finish()
    harness.check(
        len(results),
        2,
        "Should be able to inherit from an interface with "
        "[LegacyUnforgeable] properties even if we have a static operation "
        "with the same name.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Child : Parent {
              undefined foo();
            };
            interface Parent {
              [LegacyUnforgeable] readonly attribute long foo;
            };
        """
        )

        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should have thrown when shadowing unforgeable attribute on "
        "parent with operation.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Child : Parent {
              undefined foo();
            };
            interface Parent {
              [LegacyUnforgeable] undefined foo();
            };
        """
        )

        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should have thrown when shadowing unforgeable operation on "
        "parent with operation.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Child : Parent {
              attribute short foo;
            };
            interface Parent {
              [LegacyUnforgeable] readonly attribute long foo;
            };
        """
        )

        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should have thrown when shadowing unforgeable attribute on "
        "parent with attribute.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Child : Parent {
              attribute short foo;
            };
            interface Parent {
              [LegacyUnforgeable] undefined foo();
            };
        """
        )

        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should have thrown when shadowing unforgeable operation on "
        "parent with attribute.",
    )

    parser = parser.reset()
    parser.parse(
        """
            interface Child : Parent {
            };
            interface Parent {};
            interface mixin Mixin {
              [LegacyUnforgeable] readonly attribute long foo;
            };
            Parent includes Mixin;
        """
    )

    results = parser.finish()
    harness.check(
        len(results),
        4,
        "Should be able to inherit from an interface with a "
        "mixin with [LegacyUnforgeable] properties.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Child : Parent {
              undefined foo();
            };
            interface Parent {};
            interface mixin Mixin {
              [LegacyUnforgeable] readonly attribute long foo;
            };
            Parent includes Mixin;
        """
        )

        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should have thrown when shadowing unforgeable attribute "
        "of parent's consequential interface.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Child : Parent {
            };
            interface Parent : GrandParent {};
            interface GrandParent {};
            interface mixin Mixin {
              [LegacyUnforgeable] readonly attribute long foo;
            };
            GrandParent includes Mixin;
            interface mixin ChildMixin {
              undefined foo();
            };
            Child includes ChildMixin;
        """
        )

        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should have thrown when our consequential interface shadows unforgeable attribute "
        "of ancestor's consequential interface.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface Child : Parent {
            };
            interface Parent : GrandParent {};
            interface GrandParent {};
            interface mixin Mixin {
              [LegacyUnforgeable] undefined foo();
            };
            GrandParent includes Mixin;
            interface mixin ChildMixin {
              undefined foo();
            };
            Child includes ChildMixin;
        """
        )

        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should have thrown when our consequential interface shadows unforgeable operation "
        "of ancestor's consequential interface.",
    )

    parser = parser.reset()
    parser.parse(
        """
        interface iface {
          [LegacyUnforgeable] attribute long foo;
        };
    """
    )

    results = parser.finish()
    harness.check(
        len(results), 1, "Should allow writable [LegacyUnforgeable] attribute."
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface iface {
              [LegacyUnforgeable] static readonly attribute long foo;
            };
        """
        )

        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown for static [LegacyUnforgeable] attribute.")
