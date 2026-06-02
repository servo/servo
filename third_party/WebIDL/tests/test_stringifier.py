import WebIDL


def WebIDLTest(parser, harness):
    parser.parse(
        """
        interface TestStringifier {
          stringifier;
        };
    """
    )

    results = parser.finish()

    harness.ok(
        isinstance(results[0].members[0], WebIDL.IDLMethod),
        "Stringifer should be method",
    )

    parser = parser.reset()

    threw = False
    try:
        parser.parse(
            """
            interface TestStringifier {
              stringifier;
              stringifier;
            };
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow two 'stringifier;'")

    parser = parser.reset()

    threw = False
    try:
        parser.parse(
            """
            interface TestStringifier {
              stringifier;
              stringifier DOMString foo();
            };
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow a 'stringifier;' and a 'stringifier()'")

    parser = parser.reset()
    parser.parse(
        """
        interface TestStringifier {
            stringifier attribute DOMString foo;
        };
    """
    )
    results = parser.finish()
    harness.ok(
        isinstance(results[0].members[0], WebIDL.IDLAttribute),
        "Stringifier attribute should be an attribute",
    )
    stringifier = results[0].members[1]
    harness.ok(
        isinstance(stringifier, WebIDL.IDLMethod),
        "Stringifier attribute should insert a method",
    )
    harness.ok(stringifier.isStringifier(), "Inserted method should be a stringifier")

    parser = parser.reset()
    parser.parse(
        """
        interface TestStringifier {};
        interface mixin TestStringifierMixin {
            stringifier attribute DOMString foo;
        };
        TestStringifier includes TestStringifierMixin;
    """
    )
    results = parser.finish()
    harness.ok(
        isinstance(results[0].members[0], WebIDL.IDLAttribute),
        "Stringifier attribute should be an attribute",
    )
    stringifier = results[0].members[1]
    harness.ok(
        isinstance(stringifier, WebIDL.IDLMethod),
        "Stringifier attribute should insert a method",
    )
    harness.ok(stringifier.isStringifier(), "Inserted method should be a stringifier")

    parser = parser.reset()
    parser.parse(
        """
        interface TestStringifier {
            stringifier attribute USVString foo;
        };
    """
    )
    results = parser.finish()
    stringifier = results[0].members[1]
    harness.ok(
        stringifier.signatures()[0][0].isUSVString(),
        "Stringifier attributes should allow USVString",
    )

    parser = parser.reset()
    parser.parse(
        """
        interface TestStringifier {
            [Throws, NeedsSubjectPrincipal]
            stringifier attribute USVString foo;
        };
    """
    )
    results = parser.finish()
    stringifier = results[0].members[1]
    harness.ok(
        stringifier.getExtendedAttribute("Throws"),
        "Stringifier attributes should support [Throws]",
    )
    harness.ok(
        stringifier.getExtendedAttribute("NeedsSubjectPrincipal"),
        "Stringifier attributes should support [NeedsSubjectPrincipal]",
    )

    parser = parser.reset()
    parser.parse(
        """
        interface TestStringifier {
            stringifier attribute UTF8String foo;
        };
    """
    )
    results = parser.finish()
    stringifier = results[0].members[1]
    harness.ok(
        stringifier.signatures()[0][0].isUTF8String(),
        "Stringifier attributes should allow UTF8String",
    )

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface TestStringifier {
              stringifier attribute ByteString foo;
            };
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow ByteString")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface TestStringifier {
              stringifier;
              stringifier attribute DOMString foo;
            };
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow a 'stringifier;' and a stringifier attribute")

    parser = parser.reset()
    threw = False
    try:
        parser.parse(
            """
            interface TestStringifier {
              stringifier attribute DOMString foo;
              stringifier attribute DOMString bar;
            };
        """
        )
        results = parser.finish()
    except WebIDL.WebIDLError:
        threw = True

    harness.ok(threw, "Should not allow multiple stringifier attributes")
