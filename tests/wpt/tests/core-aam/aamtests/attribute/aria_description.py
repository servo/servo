# Testing: https://w3c.github.io/core-aam/#ariaDescription

TEST_HTML = "<div role='group' id='test' aria-description='hello world'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Property: Description: <value>

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_description(node) == "hello world"

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # In the accessibilityCustomContent API, expose as an AXCustomContent object with { label: AXCustomContent: object with { label: "description" } and `value` set to the description string.
#     # See also: Name Computation

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: accDescription: <value>
#     # See also: Name Computation

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: FullDescription: <value>
#     # See also: Name Computation
