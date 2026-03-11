import WebIDL


def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse(
            """
            interface SpecialMethodUniqueness1 {
              getter deleter boolean (DOMString name);
              getter boolean (DOMString name);
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse(
            """
            interface SpecialMethodUniqueness1 {
              deleter boolean (DOMString name);
              getter deleter boolean (DOMString name);
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")

    threw = False
    try:
        parser.parse(
            """
            interface SpecialMethodUniqueness1 {
              setter boolean (DOMString name);
              setter boolean (DOMString name);
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")
