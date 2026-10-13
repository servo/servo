# Testing: https://w3c.github.io/core-aam/#ariaOrientationVertical

TEST_HTML = "<div role='scrollbar' id='test' aria-orientation='vertical'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_VERTICAL
    # State: STATE_HORIZONTAL: not exposed

    node = atspi.find_node("test", session.url)
    assert "STATE_VERTICAL" in atspi.get_state_list_helper(node)
    assert "STATE_HORIZONTAL" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXOrientation: AXVerticalOrientation

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: IA2_STATE_VERTICAL
#     # State: IA2_STATE_HORIZONTAL: not exposed

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Orientation: vertical
