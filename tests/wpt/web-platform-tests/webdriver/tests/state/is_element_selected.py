# META: timeout=long

from tests.support.asserts import assert_error, assert_dialog_handled, assert_success
from tests.support.inline import inline
from tests.support.fixtures import create_dialog


alert_doc = inline("<script>window.alert()</script>")
check_doc = inline("<input id=checked type=checkbox checked/><input id=notChecked type=checkbox/>")
option_doc = inline("""<select>
                        <option id=notSelected>r-</option>
                        <option id=selected selected>r+</option>
                       </select>
                    """)


# 13.1 Is Element Selected

def test_no_browsing_context(session, create_window):
    # 13.1 step 1
    session.window_handle = create_window()
    session.close()

    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id="foo"))

    assert_error(result, "no such window")


def test_handle_prompt_dismiss(new_session, add_browser_capabilites):
    # 13.1 step 2
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "dismiss"})}})
    session.url = inline("<input id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")

    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_success(result, False)
    assert_dialog_handled(session, "dismiss #1")

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")

    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_success(result, False)
    assert_dialog_handled(session, "dismiss #2")

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")

    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_success(result, False)
    assert_dialog_handled(session, "dismiss #3")


def test_handle_prompt_accept(new_session, add_browser_capabilites):
    # 13.1 step 2
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "accept"})}})
    session.url = inline("<input id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")

    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_success(result, False)
    assert_dialog_handled(session, "dismiss #1")

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")

    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_success(result, False)
    assert_dialog_handled(session, "dismiss #2")

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")

    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_success(result, False)
    assert_dialog_handled(session, "dismiss #3")


def test_handle_prompt_missing_value(session):
    # 13.1 step 2
    session.url = inline("<input id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")

    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #1")

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")

    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #2")

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")

    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #3")


def test_element_stale(session):
    # 13.1 step 4
    session.url = check_doc
    element = session.find.css("#checked", all=False)
    session.refresh()
    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_error(result, "stale element reference")


def test_element_checked(session):
    # 13.1 step 5
    session.url = check_doc
    element = session.find.css("#checked", all=False)
    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_success(result, True)


def test_checkbox_not_selected(session):
    # 13.1 step 5
    session.url = check_doc
    element = session.find.css("#notChecked", all=False)
    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_success(result, False)


def test_element_selected(session):
    # 13.1 step 5
    session.url = option_doc
    element = session.find.css("#selected", all=False)
    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_success(result, True)


def test_element_not_selected(session):
    # 13.1 step 5
    session.url = option_doc
    element = session.find.css("#notSelected", all=False)
    result = session.transport.send("GET", "session/{session_id}/element/{element_id}/selected"
                                    .format(session_id=session.session_id,
                                            element_id=element.id))

    assert_success(result, False)
