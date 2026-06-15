import WebIDL


def WebIDLTest(parser, harness):
    def parseWithNode(test):
        parser.parse(
            """
            interface Node {};
            interface Document : Node {};
            interface Element : Node {};
            interface HTMLElement : Element {};
            """
            + test
        )

    def parseFrozenArrayAttribute(innerType):
        parseWithNode(
            f"""
            interface ReflectedAttribute {{
               [Frozen, ReflectedHTMLAttributeReturningFrozenArray]
               attribute sequence<{innerType}>? reflectedHTMLAttribute;
            }};
            """
        )

        results = parser.finish()

        harness.check(len(results), 5, "Should know about one thing")
        harness.ok(
            isinstance(results[4], WebIDL.IDLInterface), "Should have an interface here"
        )
        members = results[4].members
        harness.check(len(members), 1, "Should have one member")
        harness.ok(members[0].isAttr(), "Should have attribute")
        harness.ok(
            members[0].getExtendedAttribute(
                "ReflectedHTMLAttributeReturningFrozenArray"
            )
            is not None,
            "Should have extended attribute",
        )

    parseFrozenArrayAttribute("Element")

    parser = parser.reset()
    parseFrozenArrayAttribute("HTMLElement")

    parser = parser.reset()
    threw = False
    try:
        parseWithNode(
            """
            interface ReflectedAttribute {
              [ReflectedHTMLAttributeReturningFrozenArray]
              attribute Element? reflectedHTMLAttribute;
            };
            """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should have thrown because [ReflectedHTMLAttributeReturningFrozenArray] "
        "should only be used on attributes with a sequence<*Element> type.",
    )

    parser = parser.reset()
    threw = False
    try:
        parseWithNode(
            """
            interface ReflectedAttribute {
              [Frozen, ReflectedHTMLAttributeReturningFrozenArray]
              attribute sequence<long> reflectedHTMLAttribute;
            };
            """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should have thrown because [ReflectedHTMLAttributeReturningFrozenArray] "
        "should only be used on attributes with a sequence<*Element> type.",
    )

    parser = parser.reset()
    threw = False
    try:
        parseWithNode(
            """
            interface ReflectedAttribute {
              [Frozen, ReflectedHTMLAttributeReturningFrozenArray]
              attribute sequence<Document> reflectedHTMLAttribute;
            };
            """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should have thrown because [ReflectedHTMLAttributeReturningFrozenArray] "
        "should only be used on attributes with a sequence<*Element> type.",
    )

    parser = parser.reset()
    threw = False
    try:
        parseWithNode(
            """
            interface ReflectedAttribute {
              [ReflectedHTMLAttributeReturningFrozenArray]
              sequence<Element>? reflectedHTMLAttribute();
            };
            """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should have thrown because [ReflectedHTMLAttributeReturningFrozenArray] "
        "should only be used on attributes with a sequence<*Element> type.",
    )

    parser = parser.reset()
    threw = False
    try:
        parseWithNode(
            """
            interface ReflectedAttribute {
              [Frozen, ReflectedHTMLAttributeReturningFrozenArray, Cached, Pure]
              attribute sequence<Element>? reflectedHTMLAttribute;
            };
            """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should have thrown because [ReflectedHTMLAttributeReturningFrozenArray] "
        "should not be used together with [Cached].",
    )

    parser = parser.reset()
    threw = False
    try:
        parseWithNode(
            """
            interface ReflectedAttribute {
              [Frozen, ReflectedHTMLAttributeReturningFrozenArray, StoreInSlot, Pure]
              attribute sequence<Element>? reflectedHTMLAttribute;
            };
            """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should have thrown because [ReflectedHTMLAttributeReturningFrozenArray] "
        "should not be used together with [Cached].",
    )

    parser = parser.reset()
    threw = False
    try:
        parseWithNode(
            """
            interface ReflectedAttributeBase {
              [Frozen, ReflectedHTMLAttributeReturningFrozenArray]
              attribute sequence<Element>? reflectedHTMLAttributeBase;
            };
            interface ReflectedAttribute : ReflectedAttributeBase {
              [Frozen, ReflectedHTMLAttributeReturningFrozenArray]
              attribute sequence<Element>? reflectedHTMLAttribute;
            };
            """
        )

        parser.finish()
    except WebIDL.WebIDLError:
        threw = True
    harness.ok(
        threw,
        "Should have thrown because [ReflectedHTMLAttributeReturningFrozenArray] "
        "should not be used on the attribute of an interface that inherits from "
        "an interface that also has an attribute with "
        "[ReflectedHTMLAttributeReturningFrozenArray].",
    )
