# Testing: https://w3c.github.io/core-aam/#ariaPressedTrue

TEST_HTML = "<div role='button' id='test' aria-pressed='true'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_PRESSED

    node = atspi.find_node("test", session.url)
    assert "STATE_PRESSED" in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXValue: 1
#     # See also: button with defined value for aria-pressed

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_PRESSED
#     # See also: button with defined value for aria-pressed

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Toggle.ToggleState: On (1)
