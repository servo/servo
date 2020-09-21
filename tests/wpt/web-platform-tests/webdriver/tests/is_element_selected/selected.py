from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


check_doc = inline("""
    <input id=checked type=checkbox checked>
    <input id=notChecked type=checkbox>
    """)
option_doc = inline("""
    <select>
      <option id=notSelected>r-
      <option id=selected selected>r+
    </select>
    """)


def is_element_selected(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/selected".format(
            session_id=session.session_id,
            element_id=element_id))


def test_no_top_browsing_context(session, closed_window):
    response = is_element_selected(session, "foo")
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = is_element_selected(session, "foo")
    assert_error(response, "no such window")


def test_element_stale(session):
    session.url = check_doc
    element = session.find.css("#checked", all=False)
    session.refresh()

    result = is_element_selected(session, element.id)
    assert_error(result, "stale element reference")


def test_element_checked(session):
    session.url = check_doc
    element = session.find.css("#checked", all=False)

    result = is_element_selected(session, element.id)
    assert_success(result, True)


def test_checkbox_not_selected(session):
    session.url = check_doc
    element = session.find.css("#notChecked", all=False)

    result = is_element_selected(session, element.id)
    assert_success(result, False)


def test_element_selected(session):
    session.url = option_doc
    element = session.find.css("#selected", all=False)

    result = is_element_selected(session, element.id)
    assert_success(result, True)


def test_element_not_selected(session):
    session.url = option_doc
    element = session.find.css("#notSelected", all=False)

    result = is_element_selected(session, element.id)
    assert_success(result, False)
