from tests.support.asserts import assert_success
from tests.support.inline import inline


def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


def test_null_response_value(session):
    session.url = inline("<p>foo")
    element = session.find.css("p", all=False)

    response = element_click(session, element)
    value = assert_success(response)
    assert value is None
