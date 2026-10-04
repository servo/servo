# Testing: https://w3c.github.io/core-aam/#ariaInvalidFalse

TEST_HTML = "<div role='textbox' id='test' aria-invalid='false'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_INVALID_ENTRY: not exposed

    node = atspi.find_node("test", session.url)
    assert "STATE_INVALID_ENTRY" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXInvalid: false

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: IA2_STATE_INVALID_ENTRY: not exposed

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: IsDataValidForForm: true
