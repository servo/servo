import pytest

import webdriver.protocol as protocol

from tests.support.asserts import assert_error, assert_success


def switch_to_frame(session, frame):
    return session.transport.send(
        "POST", "session/{session_id}/frame".format(**vars(session)),
        {"id": frame},
        encoder=protocol.Encoder, decoder=protocol.Decoder,
        session=session)


def frameset(inline, *docs):
    frames = list(map(lambda doc: "<frame src='{}'></frame>".format(inline(doc)), docs))
    return "<frameset rows='{}'>\n{}</frameset>".format(len(frames) * "*,", "\n".join(frames))


def test_frame_id_webelement_no_such_element(session, iframe, inline):
    session.url = inline(iframe("<p>foo"))
    frame = session.find.css("iframe", all=False)
    frame.id = "bar"

    response = switch_to_frame(session, frame)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_frame_id_webelement_stale_element_reference(session, stale_element, as_frame):
    frame = stale_element("iframe", as_frame=as_frame)

    result = switch_to_frame(session, frame)
    assert_error(result, "stale element reference")


def test_frame_id_webelement_no_frame_element(session, inline):
    session.url = inline("<p>foo")
    no_frame = session.find.css("p", all=False)

    response = switch_to_frame(session, no_frame)
    assert_error(response, "no such frame")


@pytest.mark.parametrize("index, value", [[0, "foo"], [1, "bar"]])
def test_frame_id_webelement_frame(session, inline, index, value):
    session.url = inline(frameset(inline, "<p>foo", "<p>bar"))
    frames = session.find.css("frame")
    assert len(frames) == 2

    response = switch_to_frame(session, frames[index])
    assert_success(response)

    element = session.find.css("p", all=False)
    assert element.text == value


@pytest.mark.parametrize("index, value", [[0, "foo"], [1, "bar"]])
def test_frame_id_webelement_iframe(session, inline, iframe, index, value):
    session.url = inline("{}{}".format(iframe("<p>foo"), iframe("<p>bar")))
    frames = session.find.css("iframe")
    assert len(frames) == 2

    response = switch_to_frame(session, frames[index])
    assert_success(response)

    element = session.find.css("p", all=False)
    assert element.text == value


def test_frame_id_webelement_nested(session, inline, iframe):
    session.url = inline(iframe("{}<p>foo".format(iframe("<p>bar"))))

    expected_text = ["foo", "bar"]
    for i in range(0, len(expected_text)):
        frame_element = session.find.css("iframe", all=False)
        response = switch_to_frame(session, frame_element)
        assert_success(response)

        element = session.find.css("p", all=False)
        assert element.text == expected_text[i]


def test_frame_id_webelement_cloned_into_iframe(session, inline, iframe):
    session.url = inline(iframe("<body><p>hello world</p></body>"))

    session.execute_script("""
        const iframe = document.getElementsByTagName('iframe')[0];
        const div = document.createElement('div');
        div.innerHTML = 'I am a div created in top window and appended into the iframe';
        iframe.contentWindow.document.body.appendChild(div);
    """)

    frame = session.find.css("iframe", all=False)
    response = switch_to_frame(session, frame)
    assert_success(response)

    element = session.find.css("div", all=False)
    assert element.text == "I am a div created in top window and appended into the iframe"
