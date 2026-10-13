# Testing: https://w3c.github.io/core-aam/#ariaCheckedTrue

TEST_HTML = "<div role='checkbox' id='test' aria-checked='true'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_CHECKABLE
    # State: STATE_CHECKED

    node = atspi.find_node("test", session.url)
    assert "STATE_CHECKABLE" in atspi.get_state_list_helper(node)
    assert "STATE_CHECKED" in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXValue: 1
#     # Property: AXMenuItemMarkChar: ✓ for menuitemcheckbox and menuitemradio

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_CHECKED
#     # Object Attribute: checkable:true

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Toggle.ToggleState: On (1)
#     # Property: SelectionItem.IsSelected: True for radio and menuitemradio
