from tests.support.asserts import assert_error, assert_success, assert_dialog_handled
from tests.support.fixtures import create_dialog
from tests.support.inline import inline


def read_global(session, name):
    return session.execute_script("return %s;" % name)


def get_tag_name(session, element_id):
    return session.transport.send("GET", "session/{session_id}/element/{element_id}/name".format(
        session_id=session.session_id, element_id="foo"))


def test_handle_prompt_dismiss(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {
        "alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "dismiss"})}})
    session.url = inline("<input id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")

    result = get_tag_name(session, element.id)
    assert_success(result, "input")
    assert_dialog_handled(session, "dismiss #1")

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")

    result = get_tag_name(session, element.id)
    assert_success(result, "input")
    assert_dialog_handled(session, "dismiss #2")

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")

    result = get_tag_name(session, element.id)
    assert_success(result, "input")
    assert_dialog_handled(session, "dismiss #3")


def test_handle_prompt_accept(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {
        "alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "accept"})}})
    session.url = inline("<input id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")

    result = get_tag_name(session, element.id)
    assert_success(result, "input")
    assert_dialog_handled(session, "dismiss #1")

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")

    result = get_tag_name(session, element.id)
    assert_success(result, "input")
    assert_dialog_handled(session, "dismiss #2")

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")

    result = get_tag_name(session, element.id)
    assert_success(result, "input")
    assert_dialog_handled(session, "dismiss #3")


def test_handle_prompt_missing_value(session):
    session.url = inline("<input id=foo>")
    element = session.find.css("#foo", all=False)

    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")

    result = get_tag_name(session, element.id)
    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #1")

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")

    result = get_tag_name(session, element.id)
    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #2")

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")

    result = get_tag_name(session, element.id)
    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #3")
