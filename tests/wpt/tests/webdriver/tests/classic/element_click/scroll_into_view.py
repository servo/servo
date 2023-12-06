import pytest

from tests.support.asserts import assert_success
from tests.support.helpers import center_point


def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


def assert_one_click(session):
    """Asserts there has only been one click, and returns that."""
    clicks = session.execute_script("return window.clicks")
    assert len(clicks) == 1
    return tuple(clicks[0])


def test_scroll_into_view(session, inline):
    session.url = inline("""
        <input type=text value=Federer
        style="position: absolute; left: 0vh; top: 500vh">""")

    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_success(response)

    # Check if element clicked is scrolled into view
    assert session.execute_script("""
        let input = arguments[0];
        rect = input.getBoundingClientRect();
        return rect["top"] >= 0 && rect["left"] >= 0 &&
            (rect["top"] + rect["height"]) <= window.innerHeight &&
            (rect["left"] + rect["width"]) <= window.innerWidth;
            """, args=(element,)) is True


@pytest.mark.parametrize("offset", range(9, 0, -1))
def test_partially_visible_does_not_scroll(session, offset, inline):
    session.url = inline("""
        <style>
        body {{
          margin: 0;
          padding: 0;
        }}

        div {{
          background: blue;
          height: 200px;

          /* make N pixels visible in the viewport */
          margin-top: calc(100vh - {offset}px);
        }}
        </style>

        <div></div>

        <script>
        window.clicks = [];
        let target = document.querySelector("div");
        target.addEventListener("click", function(e) {{ window.clicks.push([e.clientX, e.clientY]); }});
        </script>
        """.format(offset=offset))
    target = session.find.css("div", all=False)
    assert session.execute_script("return window.scrollY || document.documentElement.scrollTop") == 0
    response = element_click(session, target)
    assert_success(response)
    assert session.execute_script("return window.scrollY || document.documentElement.scrollTop") == 0
    click_point = assert_one_click(session)
    assert click_point == center_point(target)
