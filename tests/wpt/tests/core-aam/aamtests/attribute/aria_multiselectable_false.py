# Testing: https://w3c.github.io/core-aam/#ariaMultiselectableFalse

TEST_HTML = "<div role='grid' id='test' aria-multiselectable='false'> <div role='row'> <div role='cell'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_MULTISELECTABLE: not exposed

    node = atspi.find_node("test", session.url)
    assert "STATE_MULTISELECTABLE" not in atspi.get_state_list_helper(node)

# Intentionally no AX API test. AX API does not surface this node or attribute.

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_MULTISELECTABLE: not exposed
#     # State: STATE_SYSTEM_EXTSELECTABLE: not exposed
#     # See also: Selection for details on accessibility events

# Intentionally no UIA test. UIA does not surface this node or attribute.
