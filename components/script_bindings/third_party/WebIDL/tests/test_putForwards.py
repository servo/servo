import WebIDL


def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse(
            """
            interface I {
              [PutForwards=B] readonly attribute long A;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface I {
              [PutForwards=B] readonly attribute J A;
            };
            interface J {
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface I {
              [PutForwards=B] attribute J A;
            };
            interface J {
              attribute long B;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface I {
              [PutForwards=B] static readonly attribute J A;
            };
            interface J {
              attribute long B;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            callback interface I {
              [PutForwards=B] readonly attribute J A;
            };
            interface J {
              attribute long B;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface I {
              [PutForwards=C] readonly attribute J A;
              [PutForwards=C] readonly attribute J B;
            };
            interface J {
              [PutForwards=D] readonly attribute K C;
            };
            interface K {
              [PutForwards=A] readonly attribute I D;
            };
        """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should have thrown.")
