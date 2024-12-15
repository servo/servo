from tests.classic.perform_actions.support.refine import get_events

PAGE_CONTENT = """
    <input></input>
    <script>
        "use strict;"

        var allEvents = { events: [] };

        const input = document.querySelector("input");
        input.focus();

        window.addEventListener("keydown", e => allEvents.events.push([e.key]));
        window.addEventListener("mousemove", e => {
            allEvents.events.push([
                e.clientX,
                e.clientY,
            ]);
        });
    </script>
"""


def test_key(session, inline, key_chain):
    session.url = inline(f"""
        <input onkeydown="window.location = '{inline(PAGE_CONTENT)}'"/>
        <script>
            const input = document.querySelector("input");
            input.focus();
        </script>
        """)

    key_chain \
        .key_down("1") \
        .key_up("1") \
        .pause(1000) \
        .key_down("2") \
        .key_up("2") \
        .perform()

    assert session.url == inline(PAGE_CONTENT)

    events = get_events(session)
    assert len(events) == 1
    assert events[0] == ["2"]


def test_pointer(session, inline, mouse_chain):
    session.url = inline(
        f"""<input onmousedown="window.location = '{inline(PAGE_CONTENT)}'"/>""")
    input = session.find.css("input", all=False)

    mouse_chain \
        .pointer_move(x=0, y=0, origin=input) \
        .pointer_down(button=0) \
        .pointer_up(button=0) \
        .pause(1000) \
        .pointer_move(x=200, y=200) \
        .perform()

    assert session.url == inline(PAGE_CONTENT)

    events = get_events(session)
    assert len(events) == 1
    assert events[0] == [200, 200]
