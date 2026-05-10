import WebIDL


def WebIDLTest(parser, harness):
    threw = False
    try:
        parser.parse(
            """
            interface A {
              legacycaller Promise<any> foo();
            };
        """
        )
        parser.finish()

    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should not allow Promise return values for legacycaller.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A {
              Promise<any> foo();
              long foo(long arg);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should not allow overloads which have both Promise and "
        "non-Promise return types.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A {
              long foo(long arg);
              Promise<any> foo();
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should not allow overloads which have both Promise and "
        "non-Promise return types.",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A {
              Promise<any>? foo();
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should not allow nullable Promise return values.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A {
              undefined foo(Promise<any>? arg);
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should not allow nullable Promise arguments.")

    parser = parser.reset()
    parser.parse(
        """
        interface A {
          Promise<any> foo();
          Promise<any> foo(long arg);
        };
    """
    )
    parser.finish()

    harness.ok(
        True, "Should allow overloads which only have Promise and return " "types."
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A {
              attribute Promise<any> attr;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should not allow writable Promise-typed attributes.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A {
              [LegacyLenientSetter] readonly attribute Promise<any> attr;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw, "Should not allow [LegacyLenientSetter] Promise-typed attributes."
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A {
              [PutForwards=bar] readonly attribute Promise<any> attr;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should not allow [PutForwards] Promise-typed attributes.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A {
              [Replaceable] readonly attribute Promise<any> attr;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should not allow [Replaceable] Promise-typed attributes.")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface A {
              [SameObject] readonly attribute Promise<any> attr;
            };
        """
        )
        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(threw, "Should not allow [SameObject] Promise-typed attributes.")
