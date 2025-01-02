import pytest

from tests.support.asserts import assert_error, assert_success


def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


def test_display_none(session, inline):
    session.url = inline("""<button style="display: none">foobar</button>""")
    element = session.find.css("button", all=False)

    response = element_click(session, element)
    assert_error(response, "element not interactable")


def test_visibility_hidden(session, inline):
    session.url = inline("""<button style="visibility: hidden">foobar</button>""")
    element = session.find.css("button", all=False)

    response = element_click(session, element)
    assert_error(response, "element not interactable")


def test_hidden(session, inline):
    session.url = inline("<button hidden>foobar</button>")
    element = session.find.css("button", all=False)

    response = element_click(session, element)
    assert_error(response, "element not interactable")


def test_disabled(session, inline):
    session.url = inline("""<button disabled>foobar</button>""")
    element = session.find.css("button", all=False)

    response = element_click(session, element)
    assert_success(response)


@pytest.mark.parametrize("transform", ["translate(100px, 100px)", "rotate(50deg)"])
def test_element_interactable_css_transform(session, inline, transform):
    # The button is transformed within the viewport.
    session.url = inline("""
        <div style="width: 500px; height: 100px; position: absolute; left: 50px; top: 200px;
            background-color: blue; transform: {transform};">
            <input type=button>
        </div>""".format(transform=transform))
    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_success(response)


@pytest.mark.parametrize("transform", ["translate(-100px, -100px)", "rotate(50deg)"])
def test_element_not_interactable_css_transform(session, inline, transform):
    # The button is transformed outside of the viewport.
    session.url = inline("""
        <div style="width: 500px; height: 100px;
            background-color: blue; transform: {transform};">
            <input type=button>
        </div>""".format(transform=transform))
    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_error(response, "element not interactable")


def test_element_not_interactable_out_of_view(session, inline):
    session.url = inline("""
        <style>
        input {
          position: absolute;
          margin-top: -100vh;
          background: red;
        }
        </style>

        <input>
        """)
    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_error(response, "element not interactable")


@pytest.mark.parametrize("tag_name", ["div", "span"])
def test_zero_sized_element(session, inline, tag_name):
    session.url = inline("<{0}></{0}>".format(tag_name))
    element = session.find.css(tag_name, all=False)

    response = element_click(session, element)
    assert_error(response, "element not interactable")


def test_element_intercepted(session, inline):
    session.url = inline("""
        <style>
        div {
          position: absolute;
          height: 100vh;
          width: 100vh;
          background: blue;
          top: 0;
          left: 0;
        }
        </style>

        <input type=button value=Roger>
        <div></div>
        """)
    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_error(response, "element click intercepted")


def test_element_intercepted_no_pointer_events(session, inline):
    session.url = inline("""<input type=button value=Roger style="pointer-events: none">""")
    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_error(response, "element click intercepted")


def test_element_not_visible_overflow_hidden(session, inline):
    session.url = inline("""
        <style>
        div {
          overflow: hidden;
          height: 50px;
          background: green;
        }

        input {
          margin-top: 100px;
          background: red;
        }
        </style>

        <div><input></div>
        """)
    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_error(response, "element not interactable")
