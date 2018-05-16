from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline

_input = inline("<input id=i1>")


def get_element_property(session, element_id, prop):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/property/{prop}".format(
            session_id=session.session_id,
            element_id=element_id,
            prop=prop))


def test_no_browsing_context(session, create_window):
    session.window_handle = create_window()
    session.close()

    result = get_element_property(session, "foo", "id")
    assert_error(result, "no such window")


def test_element_not_found(session):
    # 13.3 Step 3
    result = get_element_property(session, "foo", "id")
    assert_error(result, "no such element")


def test_element_stale(session):
    session.url = _input
    element = session.find.css("input", all=False)
    session.refresh()

    result = get_element_property(session, element.id, "id")
    assert_error(result, "stale element reference")


def test_property_non_existent(session):
    session.url = _input
    element = session.find.css("input", all=False)

    result = get_element_property(session, element.id, "foo")
    assert_success(result, None)

    assert session.execute_script("return arguments[0].foo", args=[element]) is None


def test_element(session):
    session.url = inline("<input type=checkbox>")
    element = session.find.css("input", all=False)
    element.click()
    assert session.execute_script("return arguments[0].hasAttribute('checked')", args=(element,)) is False

    result = get_element_property(session, element.id, "checked")
    assert_success(result, True)
