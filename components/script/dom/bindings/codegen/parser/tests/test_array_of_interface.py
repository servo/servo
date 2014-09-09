import WebIDL

def WebIDLTest(parser, harness):
    parser.parse("""
      interface A {
        attribute long a;
      };

      interface B {
        attribute A[] b;
      };
    """);
    parser.finish()
