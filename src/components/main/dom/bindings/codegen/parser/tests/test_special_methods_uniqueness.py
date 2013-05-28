import WebIDL

def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse("""
            interface SpecialMethodUniqueness1 {
              getter deleter boolean (DOMString name);
              getter boolean (DOMString name);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodUniqueness1 {
              deleter boolean (DOMString name);
              getter deleter boolean (DOMString name);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodUniqueness1 {
              setter creator boolean (DOMString name);
              creator boolean (DOMString name);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse("""
            interface SpecialMethodUniqueness1 {
              setter boolean (DOMString name);
              creator setter boolean (DOMString name);
            };
        """)

        results = parser.finish()
    except:
        threw = True

    harness.ok(threw, "Should have thrown.")
