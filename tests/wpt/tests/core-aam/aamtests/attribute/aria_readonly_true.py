# Testing: https://w3c.github.io/core-aam/#ariaReadonlyTrue

TEST_HTML = "<div role='checkbox' id='test' aria-readonly='true'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_READ_ONLY
    # State: STATE_EDITABLE: not exposed on text input roles
    # State: STATE_CHECKABLE: not exposed on roles supporting aria-checked
    # State: STATE_CHECKABLE: not exposed on radio descendants when used on a radiogroup

    node = atspi.find_node("test", session.url)
    assert "STATE_READ_ONLY" in atspi.get_state_list_helper(node)
    assert "STATE_EDITABLE" not in atspi.get_state_list_helper(node)
    assert "STATE_CHECKABLE" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Method: AXUIElementIsAttributeSettable(AXValue): NO

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_READONLY

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Value.IsReadOnly: true, if the element implements IValueProvider.
#     # Property: RangeValue.IsReadOnly: true, if the element implements IRangeValueProvider.
#     # Property: AriaProperties.readonly: true
