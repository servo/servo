from tests.support.asserts import assert_error


def element_click(session, element):
    return session.transport.send(
        "POST", "/session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


def test_is_stale(session, inline):
    session.url = inline("<button>foo</button>")
    button = session.find.css("button", all=False)
    session.url = inline("<button>bar</button>")

    response = element_click(session, button)
    assert_error(response, "stale element reference")
