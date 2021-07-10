def WebIDLTest(parser, harness):
    parser.parse("""
      typedef long mylong;
      typedef long? mynullablelong;
      interface Foo {
        const mylong X = 5;
        void foo(optional mynullablelong arg = 7);
        void bar(optional mynullablelong arg = null);
        void baz(mylong arg);
      };
    """)

    results = parser.finish()

    harness.check(results[2].members[1].signatures()[0][1][0].type.name, "LongOrNull",
                  "Should expand typedefs")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          typedef long? mynullablelong;
          interface Foo {
            void foo(mynullablelong? Y);
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown on nullable inside nullable arg.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          typedef long? mynullablelong;
          interface Foo {
            const mynullablelong? X = 5;
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown on nullable inside nullable const.")
    
    parser = parser.reset()
    threw = False
    try:
        parser.parse("""
          interface Foo {
            const mynullablelong? X = 5;
          };
          typedef long? mynullablelong;
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw,
               "Should have thrown on nullable inside nullable const typedef "
               "after interface.")

    parser = parser.reset()
    parser.parse("""
      interface Foo {
        const mylong X = 5;
      };
      typedef long mylong;
    """)

    results = parser.finish()

    harness.check(results[0].members[0].type.name, "Long",
                  "Should expand typedefs that come before interface")
