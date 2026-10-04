# Testing: https://w3c.github.io/core-aam/#ariaReadonlyFalse

TEST_HTML = "<div role='searchbox' id='test' aria-readonly='false'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_READ_ONLY: not exposed

    node = atspi.find_node("test", session.url)
    assert "STATE_READ_ONLY" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Method: AXUIElementIsAttributeSettable(AXValue): YES

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_READONLY: not exposed
#     # State: IA2_STATE_EDITABLE

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Value.IsReadOnly: false, if the element implements IValueProvider.
#     # Property: RangeValue.IsReadOnly: false, if the element implements IRangeValueProvider.
#     # Property: AriaProperties.readonly: false
