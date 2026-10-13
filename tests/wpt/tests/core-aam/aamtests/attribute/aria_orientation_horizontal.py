# Testing: https://w3c.github.io/core-aam/#ariaOrientationHorizontal

TEST_HTML = "<div role='scrollbar' id='test' aria-orientation='horizontal'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_HORIZONTAL
    # State: STATE_VERTICAL: not exposed

    node = atspi.find_node("test", session.url)
    assert "STATE_HORIZONTAL" in atspi.get_state_list_helper(node)
    assert "STATE_VERTICAL" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXOrientation: AXHorizontalOrientation

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: IA2_STATE_HORIZONTAL
#     # State: IA2_STATE_VERTICAL: not exposed

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Orientation: horizontal
