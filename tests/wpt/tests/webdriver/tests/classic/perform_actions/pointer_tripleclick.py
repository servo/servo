import math

lots_of_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor "\
               "incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud "\
               "exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat."


def test_tripleclick_at_coordinates(session, mouse_chain, inline):
    """
        This test does a triple click on a coordinate. On desktop platforms
        this will select a paragraph. On mobile this will not have the same
        desired outcome as taps are handled differently on mobile.
    """
    session.url = inline("""<div>{}</div>""".format(lots_of_text))
    div = session.find.css("div", all=False)
    div_rect = div.rect
    div_centre = {
        "x": math.floor(div_rect["x"] + div_rect["width"] / 2),
        "y": math.floor(div_rect["y"] + div_rect["height"] / 2)
    }
    mouse_chain \
        .pointer_move(div_centre["x"], div_centre["y"]) \
        .click() \
        .click() \
        .click() \
        .perform()

    actual_text = session.execute_script("return document.getSelection().toString();")

    assert actual_text == lots_of_text
