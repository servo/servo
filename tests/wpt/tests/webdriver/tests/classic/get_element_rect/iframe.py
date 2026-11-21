import pytest

from tests.support.classic.asserts import assert_success
from tests.support.classic.helpers import element_rect


def get_element_rect(session, element_id):
    return session.transport.send(
        "GET",
        "session/{session_id}/element/{element_id}/rect".format(
            session_id=session.session_id,
            element_id=element_id,
        )
    )


IFRAME_STYLE = "<style>iframe { position:absolute; left:50px; top:70px; " \
               "width:200px; height:200px; border:0; }</style>"
INNER = "<style>body{margin:0}#box{position:absolute;left:10px;top:20px;" \
        "width:30px;height:40px}</style><div id='box'></div>"


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_element_rect_in_frame(
    session, inline, iframe, domain
):
    session.url = inline("{0}{1}".format(IFRAME_STYLE, iframe(INNER, domain=domain)))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    box = session.find.css("#box", all=False)
    response = get_element_rect(session, box.id)
    rect = assert_success(response)

    assert rect["width"] == 30    # size must survive the transform intact.
    assert rect["height"] == 40

    assert element_rect(session, box) == {
        "x": rect["x"], "y": rect["y"],
        "width": rect["width"], "height": rect["height"],
    }

SCROLLED_IFRAME_STYLE = "<style>body{margin:0;height:2000px}" \
                        "iframe{position:absolute;left:100px;top:150px;" \
                        "width:300px;height:200px;border:0}</style>"
SCROLLED_INNER = "<style>body{margin:0;height:1000px}#box{position:absolute;left:10px;" \
                 "top:520px;width:30px;height:40px}</style><div id='box'></div>"


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_element_rect_in_frame_with_scrolled_main_frame(session, inline, iframe, domain):
    session.url = inline("{0}{1}".format(SCROLLED_IFRAME_STYLE, iframe(SCROLLED_INNER, domain=domain)))

    session.execute_script("window.scrollTo(0, 100)")
    assert session.execute_script("return window.scrollY") == 100

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    box = session.find.css("#box", all=False)
    response = get_element_rect(session, box.id)
    rect = assert_success(response)

    # Coordinates within the frame's own document, unmoved by the iframe's offset in the main
    # frame or by the main frame's scroll position.
    assert rect == {"x": 10, "y": 520, "width": 30, "height": 40}

    assert element_rect(session, box) == {
        "x": rect["x"], "y": rect["y"],
        "width": rect["width"], "height": rect["height"],
    }
