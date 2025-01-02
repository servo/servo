from tests.support.asserts import assert_error


def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


def test_file_upload_state(session, inline):
    session.url = inline("<input type=file>")

    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_error(response, "invalid argument")
