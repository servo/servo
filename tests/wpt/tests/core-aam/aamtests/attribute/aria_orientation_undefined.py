# Testing: https://w3c.github.io/core-aam/#ariaOrientationUndefined

TEST_HTML = "<div role='radiogroup' id='test' aria-orientation='undefined'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_VERTICAL: not exposed
    # State: STATE_HORIZONTAL: not exposed

    node = atspi.find_node("test", session.url)
    assert "STATE_VERTICAL" not in atspi.get_state_list_helper(node)
    assert "STATE_HORIZONTAL" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXOrientation: AXUnknownOrientation

# Intentionally no IA2 test. IA2 does not surface this node or attribute.

# Intentionally no UIA test. UIA does not surface this node or attribute.
