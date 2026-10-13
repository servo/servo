# Testing: https://w3c.github.io/core-aam/#ariaGrabbedTrue

TEST_HTML = "<div role='group' id='test' aria-grabbed='true'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: grabbed:true

    node = atspi.find_node("test", session.url)
    assert "grabbed:true" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXGrabbed: YES

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: grabbed:true

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.grabbed: true
