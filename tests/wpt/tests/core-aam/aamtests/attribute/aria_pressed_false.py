# Testing: https://w3c.github.io/core-aam/#ariaPressedFalse

TEST_HTML = "<div role='button' id='test' aria-pressed='false'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_PRESSED: not exposed

    node = atspi.find_node("test", session.url)
    assert "STATE_PRESSED" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXValue: 0
#     # See also: button with defined value for aria-pressed

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_PRESSED: not exposed
#     # See also: button with defined value for aria-pressed

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Toggle.ToggleState: Off (3)
