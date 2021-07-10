from tests.support.asserts import assert_error, assert_success


def get_element_text(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/text".format(
            session_id=session.session_id,
            element_id=element_id))


def test_no_top_browsing_context(session, closed_window):
    original_handle, element = closed_window
    response = get_element_text(session, element.id)
    assert_error(response, "no such window")
    response = get_element_text(session, "foo")
    assert_error(response, "no such window")

    session.window_handle = original_handle
    response = get_element_text(session, element.id)
    assert_error(response, "no such element")


def test_no_browsing_context(session, closed_frame):
    response = get_element_text(session, "foo")
    assert_error(response, "no such window")


def test_getting_text_of_a_non_existant_element_is_an_error(session, inline):
    session.url = inline("""<body>Hello world</body>""")

    result = get_element_text(session, "foo")
    assert_error(result, "no such element")


def test_read_element_text(session, inline):
    session.url = inline("Before f<span id='id'>oo</span> after")
    element = session.find.css("#id", all=False)

    result = get_element_text(session, element.id)
    assert_success(result, "oo")
