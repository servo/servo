# Testing: https://w3c.github.io/aria/core-aam/#event-aria-invalid

TEST_HTML = "<div id='target' role='textbox' aria-invalid='false'>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # object:state-changed:invalid_entry

    node = atspi.find_node("target", session.url)
    assert "STATE_INVALID_ENTRY" not in atspi.get_state_list_helper(node)

    def toggle_aria_invalid():
        session.execute_script(
            "target.ariaInvalid = target.ariaInvalid === 'true' ? 'false' : 'true'",
        )

    event = atspi.expect_event(
        "object:state-changed:invalid-entry", dom_id="target",
        action=toggle_aria_invalid,
    )

    assert event.detail1 == 1
    assert "STATE_INVALID_ENTRY" in atspi.get_state_list_helper(node)

    event = atspi.expect_event(
        "object:state-changed:invalid-entry", dom_id="target",
        action=toggle_aria_invalid,
    )

    assert event.detail1 == 0
    assert "STATE_INVALID_ENTRY" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # AXInvalidStatusChanged

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # EVENT_OBJECT_STATECHANGE

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # PropertyChangedEvent: AriaProperties, IsDataValidForForm
