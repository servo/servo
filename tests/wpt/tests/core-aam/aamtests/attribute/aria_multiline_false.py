# Testing: https://w3c.github.io/core-aam/#ariaMultilineFalse

TEST_HTML = "<div role='textbox' id='test' aria-multiline='false'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_SINGLE_LINE
    # State: STATE_MULTI_LINE: not exposed

    node = atspi.find_node("test", session.url)
    assert "STATE_SINGLE_LINE" in atspi.get_state_list_helper(node)
    assert "STATE_MULTI_LINE" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Not mapped
#     # See also: textbox in the Role Mapping Tables

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: IA2_STATE_SINGLE_LINE
#     # State: IA2_STATE_MULTI_LINE: not exposed

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Not mapped
