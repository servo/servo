# Testing: https://w3c.github.io/core-aam/#ariaDisabledFalse

TEST_HTML = "<div role='button' id='test' aria-disabled='false'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_ENABLED

    node = atspi.find_node("test", session.url)
    assert "STATE_ENABLED" in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXEnabled: YES

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_UNAVAILABLE: not exposed

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: IsEnabled: true
