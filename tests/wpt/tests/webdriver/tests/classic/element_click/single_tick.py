from tests.support.classic.asserts import assert_success


def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


def test_element_click_dispatches_actions_in_a_single_tick(session, inline):
    """Element click dispatches its pointer actions in a single tick.

    Per https://w3c.github.io/webdriver/#dfn-dispatch-a-list-of-actions
    the pointer move/down/up actions of element click form a single tick,
    so the remote end must not spin the event loop between dispatching
    them. We detect an event-loop turn by scheduling a macrotask in a
    mousedown listener and checking, from a mouseup listener, that it has
    not run yet.
    """
    session.url = inline("""
        <div id="target">click me</div>
        <script>
            window.__result = null;
            window.addEventListener("mousedown", () => {
                window.__timer_ran = false;
                setTimeout(() => { window.__timer_ran = true; }, 0);
            });
            window.addEventListener("mouseup", () => {
                window.__ran_when_mouseup = window.__timer_ran;
            });
        </script>
    """)
    element = session.find.css("#target", all=False)

    response = element_click(session, element)
    assert_success(response)

    timer_ran = session.execute_script("return window.__timer_ran;")
    assert timer_ran

    ran_when_mouseup = session.execute_script("return window.__ran_when_mouseup;")
    assert not ran_when_mouseup
