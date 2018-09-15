def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface Foo {
              [CEReactions(DOMString a)] void foo(boolean arg2);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown for [CEReactions] with an argument")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface Foo {
              [CEReactions(DOMString b)] readonly attribute boolean bar;
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown for [CEReactions] with an argument")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface Foo {
              [CEReactions] attribute boolean bar;
            };
        """)

        results = parser.finish()
    except Exception, e:
        harness.ok(False, "Shouldn't have thrown for [CEReactions] used on writable attribute. %s" % e)
        threw = True

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface Foo {
              [CEReactions] void foo(boolean arg2);
            };
        """)

        results = parser.finish()
    except Exception, e:
        harness.ok(False, "Shouldn't have thrown for [CEReactions] used on regular operations. %s" % e)
        threw = True

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            interface Foo {
              [CEReactions] readonly attribute boolean A;
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown for [CEReactions] used on a readonly attribute")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
            [CEReactions]
            interface Foo {
            }
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown for [CEReactions] used on a interface")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          interface Foo {
            [CEReactions] getter any(DOMString name);
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should have thrown for [CEReactions] used on a named getter")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          interface Foo {
            [CEReactions] legacycaller double compute(double x);
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should have thrown for [CEReactions] used on a legacycaller")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          interface Foo {
            [CEReactions] stringifier DOMString ();
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should have thrown for [CEReactions] used on a stringifier")

