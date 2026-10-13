# Testing: https://w3c.github.io/core-aam/#ariaDisabledTrue

TEST_HTML = "<div role='group' id='test' aria-disabled='true'> <div role='checkbox' id='checkbox' tabindex='0'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_ENABLED: not exposed

    node = atspi.find_node("test", session.url)
    assert "STATE_ENABLED" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXEnabled: NO

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_UNAVAILABLE
#     # State: STATE_SYSTEM_UNAVAILABLE: on all descendants with STATE_SYSTEM_FOCUSABLE

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: IsEnabled: false
