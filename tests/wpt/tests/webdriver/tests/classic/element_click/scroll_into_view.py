import pytest

from tests.support.asserts import assert_success
from tests.support.helpers import center_point


def element_click(session, element):
    return session.transport.send(
        "POST",
        "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id, element_id=element.id
        ),
    )


def assert_one_click(session):
    """Asserts there has only been one click, and returns that."""
    clicks = session.execute_script("return window.clicks")
    assert len(clicks) == 1
    return clicks[0]


def test_scroll_into_view(session, inline):
    original_handle = session.window_handle

    # Use a new tab to close the virtual keyboard that might have opened by
    # clicking the input field.
    new_handle = session.new_window()

    session.window_handle = new_handle

    session.url = inline(
        """
        <style>
            .scroll-container {
                display: block;
                overflow-y: scroll;
                scroll-behavior: smooth;
                height: 200px;
            }

            input {
                margin-top: 2000vh;
            }
        </style>

        <div class="scroll-container">
            <input type="text" size="20" value="foo"></input>
        </div>

        <script>
            window.clicks = [];
            const target = document.querySelector("input");
            target.addEventListener("click", function(e) {
                window.clicks.push({
                    "target": e.target,
                    "clientX": e.clientX,
                    "clientY": e.clientY,
                });
            });
        </script>
        """
    )

    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_success(response)

    event_data = assert_one_click(session)
    assert event_data["target"] == element

    # Check if element clicked is scrolled into view
    assert (
        session.execute_script(
            """
        let input = arguments[0];
        rect = input.getBoundingClientRect();
        return rect.top >= 0 && rect.left >= 0 &&
            Math.floor(rect.bottom) <= window.innerHeight &&
            Math.floor(rect.right) <= window.innerWidth;
            """,
            args=(element,),
        )
        is True
    )

    session.window.close()

    session.window_handle = original_handle


@pytest.mark.parametrize("offset", range(9, 0, -1))
def test_partially_visible_does_not_scroll(session, offset, inline):
    session.url = inline(
        f"""
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
            const target = document.querySelector("div");
            target.addEventListener("click", function(e) {{
                window.clicks.push({{
                    "target": e.target,
                    "clientX": e.clientX,
                    "clientY": e.clientY,
                }});
            }});
        </script>
        """
    )

    assert (
        session.execute_script(
            "return window.scrollY || document.documentElement.scrollTop"
        )
        == 0
    )

    element = session.find.css("div", all=False)
    response = element_click(session, element)
    assert_success(response)

    assert (
        session.execute_script(
            "return window.scrollY || document.documentElement.scrollTop"
        )
        == 0
    )

    event_data = assert_one_click(session)
    assert (event_data["clientX"], event_data["clientY"]) == pytest.approx(
        center_point(element), abs=1.0
    )
