# Testing: https://w3c.github.io/core-aam/#ariaPressedMixed

TEST_HTML = "<div role='button' id='test' aria-pressed='mixed'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_INDETERMINATE

    node = atspi.find_node("test", session.url)
    assert "STATE_INDETERMINATE" in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXValue: 2
#     # See also: button with defined value for aria-pressed

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_MIXED
#     # See also: button with defined value for aria-pressed

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Toggle.ToggleState: Indeterminate (2)
