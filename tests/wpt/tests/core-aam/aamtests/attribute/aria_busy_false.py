# Testing: https://w3c.github.io/core-aam/#ariaBusyFalse

TEST_HTML = "<div role='group' id='test' aria-busy='false'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_BUSY: not exposed

    node = atspi.find_node("test", session.url)
    assert "STATE_BUSY" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXElementBusy: NO

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: STATE_SYSTEM_BUSY: not exposed

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AriaProperties.busy: false
