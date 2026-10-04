# Testing: https://w3c.github.io/core-aam/#ariaExpandedFalse

TEST_HTML = "<div role='button' id='test' aria-expanded='false'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_EXPANDABLE
    # State: STATE_EXPANDED: not exposed

    node = atspi.find_node("test", session.url)
    assert "STATE_EXPANDABLE" in atspi.get_state_list_helper(node)
    assert "STATE_EXPANDED" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXExpanded: NO

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_COLLAPSED

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: ExpandCollapse.ExpandCollapseState: Collapsed
