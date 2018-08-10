from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def get_element_text(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/text".format(
            session_id=session.session_id,
            element_id=element_id))


def test_no_browsing_context(session, closed_window):
    response = get_element_text(session, "foo")
    assert_error(response, "no such window")


def test_getting_text_of_a_non_existant_element_is_an_error(session):
    session.url = inline("""<body>Hello world</body>""")

    result = get_element_text(session, "foo")
    assert_error(result, "no such element")


def test_read_element_text(session):
    session.url = inline("""
        <body>
          Noise before <span id='id'>This has an ID</span>. Noise after
        </body>""")

    element = session.find.css("#id", all=False)

    result = get_element_text(session, element.id)
    assert_success(result, "This has an ID")
