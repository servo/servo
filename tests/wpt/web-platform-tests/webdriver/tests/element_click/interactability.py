import pytest

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


def test_display_none(session):
    session.url = inline("""<button style="display: none">foobar</button>""")
    element = session.find.css("button", all=False)

    response = element_click(session, element)
    assert_error(response, "element not interactable")


def test_visibility_hidden(session):
    session.url = inline("""<button style="visibility: hidden">foobar</button>""")
    element = session.find.css("button", all=False)

    response = element_click(session, element)
    assert_error(response, "element not interactable")


def test_hidden(session):
    session.url = inline("<button hidden>foobar</button>")
    element = session.find.css("button", all=False)

    response = element_click(session, element)
    assert_error(response, "element not interactable")


def test_disabled(session):
    session.url = inline("""<button disabled>foobar</button>""")
    element = session.find.css("button", all=False)

    response = element_click(session, element)
    assert_success(response)


@pytest.mark.parametrize("transform", ["translate(-100px, -100px)", "rotate(50deg)"])
def test_element_not_interactable_css_transform(session, transform):
    session.url = inline("""
        <div style="width: 500px; height: 100px;
            background-color: blue; transform: {transform};">
            <input type=button>
        </div>""".format(transform=transform))
    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_error(response, "element not interactable")


def test_element_not_interactable_out_of_view(session):
    session.url = inline("""
        <div style="width: 500px; height: 100px;
            position: absolute; left: 0px; top: -150px; background-color: blue;">
        </div>""")
    element = session.find.css("div", all=False)
    response = element_click(session, element)
    assert_error(response, "element not interactable")


@pytest.mark.parametrize("tag_name", ["div", "span"])
def test_zero_sized_element(session, tag_name):
    session.url = inline("<{0}></{0}>".format(tag_name))
    element = session.find.css(tag_name, all=False)

    response = element_click(session, element)
    assert_error(response, "element not interactable")


def test_element_intercepted(session):
    session.url = inline("""
        <input type=button value=Roger style="position: absolute; left: 10px; top: 10px">
        <div style="position: absolute; height: 100px; width: 100px; background: rgba(255,0,0,.5); left: 10px; top: 5px"></div>""")

    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_error(response, "element click intercepted")


def test_element_intercepted_no_pointer_events(session):
    session.url = inline("""<input type=button value=Roger style="pointer-events: none">""")

    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_error(response, "element click intercepted")


def test_element_not_visible_overflow_hidden(session):
    session.url = inline("""
        <div style="position: absolute; height: 50px; width: 100px; background: rgba(255,0,0,.5); left: 10px; top: 50px; overflow: hidden">
            ABCDEFGHIJKLMNOPQRSTUVWXYZ
            <input type=text value=Federer style="position: absolute; top: 50px; left: 10px;">
        </div>""")

    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_error(response, "element not interactable")
