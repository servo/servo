def WebIDLTest(parser, harness):
    parser.parse("""
      [Global]
      interface Foo : Bar {
        getter any(DOMString name);
      };
      interface Bar {};
    """)

    results = parser.finish()

    harness.ok(results[0].isOnGlobalProtoChain(),
               "[Global] interface should be on global's proto chain")
    harness.ok(results[1].isOnGlobalProtoChain(),
               "[Global] interface should be on global's proto chain")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          [Global]
          interface Foo {
            getter any(DOMString name);
            setter void(DOMString name, any arg);
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should have thrown for [Global] used on an interface with a "
               "named setter")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          [Global]
          interface Foo {
            getter any(DOMString name);
            creator void(DOMString name, any arg);
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should have thrown for [Global] used on an interface with a "
               "named creator")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          [Global]
          interface Foo {
            getter any(DOMString name);
            deleter void(DOMString name);
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should have thrown for [Global] used on an interface with a "
               "named deleter")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          [Global, OverrideBuiltins]
          interface Foo {
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should have thrown for [Global] used on an interface with a "
               "[OverrideBuiltins]")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          [Global]
          interface Foo : Bar {
          };
          [OverrideBuiltins]
          interface Bar {
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should have thrown for [Global] used on an interface with an "
               "[OverrideBuiltins] ancestor")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          [Global]
          interface Foo {
          };
          interface Bar : Foo {
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should have thrown for [Global] used on an interface with a "
               "descendant")
