# Testing: https://w3c.github.io/core-aam/#ariaCurrentUnrecognizedValue

TEST_HTML = "<div role='group' id='test' aria-current='foo'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: current:true
    # State: STATE_ACTIVE

    node = atspi.find_node("test", session.url)
    assert "current:true" in atspi.Accessible.get_attributes_as_array(node)
    assert "STATE_ACTIVE" in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXARIACurrent: true

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Object Attribute: current:true

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.current: true
