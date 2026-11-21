"""Page builders and assertions shared by element_click/iframe.py and perform_actions/iframe.py.

Element Click and an element-relative Perform Actions origin both ask the browser for an
element's in-view center point, which has to be translated out of the frame's coordinate space
and into the main frame's before a synthesized event can be dispatched there. The helpers here
build a frame whose geometry makes an error in that translation visible, and check where the
event actually landed.
"""

import pytest

TARGET_LEFT = 40
TARGET_WIDTH = 60
TARGET_HEIGHT = 40
# How far below the frame's scroll position the target sits. Keeping this constant means the
# target is in view whatever the frame is scrolled to, so the command's "scroll into view" step
# is a no-op and cannot perturb the geometry under test.
TARGET_TOP_IN_VIEW = 20
SUBFRAME_HEIGHT = 1000


def target_document(frame_scroll):
    """A tall document with a target placed TARGET_TOP_IN_VIEW below `frame_scroll`."""
    return """
        <style>
          body {{ margin: 0; height: {height}px; }}
          #target {{
            position: absolute;
            left: {left}px;
            top: {top}px;
            width: {width}px;
            height: {target_height}px;
            background: blue;
          }}
        </style>
        <div id="target"></div>
        <script>
          window.clicks = [];
          window.addEventListener("click", e => window.clicks.push([e.clientX, e.clientY]));
        </script>
        """.format(
            height=SUBFRAME_HEIGHT,
            left=TARGET_LEFT,
            top=frame_scroll + TARGET_TOP_IN_VIEW,
            width=TARGET_WIDTH,
            target_height=TARGET_HEIGHT)


def containing_document(frame_markup, left, top, width, height, body_height, frame_style=""):
    """A document positioning a single <iframe> at a known offset."""
    return """
        <style>
          body {{ margin: 0; height: {body_height}px; }}
          iframe {{
            position: absolute;
            border: 0;
            left: {left}px;
            top: {top}px;
            width: {width}px;
            height: {height}px;
            {frame_style}
          }}
        </style>
        {frame}
        """.format(
            body_height=body_height,
            left=left,
            top=top,
            width=width,
            height=height,
            frame_style=frame_style,
            frame=frame_markup)


def scroll_to(session, y):
    session.execute_script("window.scrollTo(0, arguments[0])", args=(y,))
    scroll_y = session.execute_script("return window.scrollY")
    assert scroll_y == y, "expected window.scrollY to be {}, got {}".format(y, scroll_y)


def in_view_center_point(session):
    """The target's in-view center point in the subframe's own client coordinates, which is what
    an event in that frame reports as clientX/clientY. Deliberately independent of any ancestor's
    scroll position, so it stays valid even if the main frame scrolls."""
    return tuple(session.execute_script("""
        const rect = document.querySelector("#target").getBoundingClientRect();
        const left = Math.max(0, rect.left);
        const right = Math.min(window.innerWidth, rect.right);
        const top = Math.max(0, rect.top);
        const bottom = Math.min(window.innerHeight, rect.bottom);
        return [Math.floor((left + right) / 2), Math.floor((top + bottom) / 2)];
        """))


def assert_clicked_at(session, expected, frame_scroll=0):
    # If the frame itself scrolled, `expected` no longer describes where the target is and the
    # comparison below would be meaningless.
    scroll_y = session.execute_script("return window.scrollY")
    assert scroll_y == frame_scroll, (
        "expected the frame to stay scrolled to {}, got {}".format(frame_scroll, scroll_y))

    clicks = session.execute_script("return window.clicks")
    assert len(clicks) == 1, "expected exactly one click in the frame, got {}".format(clicks)

    click = tuple(clicks[0])
    # Coordinates are floored in several places on the way out, so allow a pixel of slack.
    assert click == pytest.approx(expected, abs=1), (
        "expected a click at {}, got one at {}".format(expected, click))
