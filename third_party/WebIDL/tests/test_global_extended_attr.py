import WebIDL


def WebIDLTest(parser, harness):
    parser.parse(
        """
      [Global=Foo, Exposed=Foo]
      interface Foo : Bar {
        getter any(DOMString name);
      };
      [Exposed=Foo]
      interface Bar {};
    """
    )

    results = parser.finish()

    harness.ok(
        results[0].isOnGlobalProtoChain(),
        "[Global] interface should be on global's proto chain",
    )
    harness.ok(
        results[1].isOnGlobalProtoChain(),
        "[Global] interface should be on global's proto chain",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          [Global=Foo, Exposed=Foo]
          interface Foo {
            getter any(DOMString name);
            setter undefined(DOMString name, any arg);
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should have thrown for [Global] used on an interface with a " "named setter",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          [Global=Foo, Exposed=Foo]
          interface Foo {
            getter any(DOMString name);
            deleter undefined(DOMString name);
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should have thrown for [Global] used on an interface with a " "named deleter",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          [Global=Foo, LegacyOverrideBuiltIns, Exposed=Foo]
          interface Foo {
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should have thrown for [Global] used on an interface with a "
        "[LegacyOverrideBuiltIns]",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          [Global=Foo, Exposed=Foo]
          interface Foo : Bar {
          };
          [LegacyOverrideBuiltIns, Exposed=Foo]
          interface Bar {
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should have thrown for [Global] used on an interface with an "
        "[LegacyOverrideBuiltIns] ancestor",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          [Global=Foo, Exposed=Foo]
          interface Foo {
          };
          [Exposed=Foo]
          interface Bar : Foo {
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should have thrown for [Global] used on an interface with a " "descendant",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
          [Global, Exposed=Foo]
          interface Foo {
          };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(
        threw,
        "Should have thrown for [Global] without a right hand side value",
    )
