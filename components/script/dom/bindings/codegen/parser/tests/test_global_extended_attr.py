def WebIDLTest(parser, harness):
    parser.parse(
        """
      [Global, Exposed=Foo]
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
          [Global, Exposed=Foo]
          interface Foo {
            getter any(DOMString name);
            setter undefined(DOMString name, any arg);
          };
        """
        )
        results = parser.finish()
    except:
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
          [Global, Exposed=Foo]
          interface Foo {
            getter any(DOMString name);
            deleter undefined(DOMString name);
          };
        """
        )
        results = parser.finish()
    except:
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
          [Global, LegacyOverrideBuiltIns, Exposed=Foo]
          interface Foo {
          };
        """
        )
        results = parser.finish()
    except:
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
          [Global, Exposed=Foo]
          interface Foo : Bar {
          };
          [LegacyOverrideBuiltIns, Exposed=Foo]
          interface Bar {
          };
        """
        )
        results = parser.finish()
    except:
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
          [Global, Exposed=Foo]
          interface Foo {
          };
          [Exposed=Foo]
          interface Bar : Foo {
          };
        """
        )
        results = parser.finish()
    except:
        threw = True

    harness.ok(
        threw,
        "Should have thrown for [Global] used on an interface with a " "descendant",
    )
