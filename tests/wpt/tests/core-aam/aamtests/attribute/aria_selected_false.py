# Testing: https://w3c.github.io/core-aam/#ariaSelectedFalse

TEST_HTML = "<div role='grid'> <div role='row'> <div role='gridcell' id='test' aria-selected='false'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_SELECTABLE
    # State: STATE_SELECTED: not exposed

    node = atspi.find_node("test", session.url)
    assert "STATE_SELECTABLE" in atspi.get_state_list_helper(node)
    assert "STATE_SELECTED" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXSelected: NO

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_SELECTABLE
#     # State: STATE_SYSTEM_SELECTED: not exposed
#     # See also: Selection for details on accessibility events

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: SelectionItem.IsSelected: false
