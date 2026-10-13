# Testing: https://w3c.github.io/core-aam/#ariaMultiselectableTrue

TEST_HTML = "<div role='grid' id='test' aria-multiselectable='true'> <div role='row'> <div role='cell'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_MULTISELECTABLE

    node = atspi.find_node("test", session.url)
    assert "STATE_MULTISELECTABLE" in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXIsMultiSelectable: YES
#     # See also: Selection for details on accessibility events

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_MULTISELECTABLE
#     # State: STATE_SYSTEM_EXTSELECTABLE
#     # See also: Selection for details on accessibility events

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Selection.CanSelectMultiple: true
#     # See also: Selection for details on accessibility events
