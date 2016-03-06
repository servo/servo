def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
          dictionary Foo {
            short a;
          };

          dictionary Foo1 {
            Foo[] b;
          };
        """)
        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Array must not contain dictionary "
                      "as element type.")
