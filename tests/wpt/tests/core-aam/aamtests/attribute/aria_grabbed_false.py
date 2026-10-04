# Testing: https://w3c.github.io/core-aam/#ariaGrabbedFalse

TEST_HTML = "<div role='group' id='test' aria-grabbed='false'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: grabbed:false

    node = atspi.find_node("test", session.url)
    assert "grabbed:false" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXGrabbed: NO

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: grabbed:false

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.grabbed: false
