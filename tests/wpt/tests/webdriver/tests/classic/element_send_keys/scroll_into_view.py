import pytest

from tests.support.asserts import assert_success
from tests.support.helpers import is_element_in_viewport


def element_send_keys(session, element, text):
    return session.transport.send(
        "POST",
        "/session/{session_id}/element/{element_id}/value".format(
            session_id=session.session_id, element_id=element.id
        ),
        {"text": text},
    )


def get_bounding_client_top(session, element):
    return session.execute_script(
        "return arguments[0].getBoundingClientRect().top", args=(element,)
    )


def test_element_outside_of_not_scrollable_viewport(session, inline):
    session.url = inline('<input style="position: relative; left: -9999px;">')
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, "foo")
    assert_success(response)

    assert not is_element_in_viewport(session, element)


def test_element_outside_of_scrollable_viewport(session, inline):
    session.url = inline('<input style="margin-top: 102vh;">')
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, "foo")
    assert_success(response)

    assert is_element_in_viewport(session, element)


def test_contenteditable_element_outside_of_scrollable_viewport(session, inline):
    session.url = inline('<div contenteditable style="margin-top: 102vh;"></div>')
    element = session.find.css("div", all=False)

    response = element_send_keys(session, element, "foo")
    assert_success(response)

    assert is_element_in_viewport(session, element)


@pytest.mark.parametrize(
    "scrolloptions",
    [
        "{block: 'start'}",
        "{block: 'center'}",
        "{block: 'end'}",
        "{block: 'nearest'}",
    ],
)
def test_element_already_in_viewport(session, inline, scrolloptions):
    session.url = inline(
        '<div style="height: 600vh;"><input style="position: absolute; top:300vh;"></div>'
    )
    element = session.find.css("input", all=False)
    session.execute_script(
        f"arguments[0].scrollIntoView({scrolloptions})", args=(element,)
    )

    assert is_element_in_viewport(session, element)
    initial_top = get_bounding_client_top(session, element)

    response = element_send_keys(session, element, "cat")
    assert_success(response)

    # Account for potential rounding errors.
    # TODO It may be possible to remove the fuzzy check once bug 1852884 and bug 1774315 are fixed.
    assert (
        (initial_top - 1)
        <= get_bounding_client_top(session, element)
        <= (initial_top + 1)
    )
    assert is_element_in_viewport(session, element)


@pytest.mark.parametrize(
    ("scrolloptions", "scrollby"),
    [
        ("{block: 'start'}", 25),
        ("{block: 'end'}", -25),
    ],
    ids=["Just above viewport", "Just below viewport"],
)
def test_element_just_outside_viewport(session, inline, scrolloptions, scrollby):
    session.url = inline(
        '<div style="height: 600vh;"><input style="position: absolute; top:300vh; height:30px"></div>'
    )
    element = session.find.css("input", all=False)
    session.execute_script(
        f"""
        arguments[0].scrollIntoView({scrolloptions});
        window.scrollBy(0, {scrollby});
        """,
        args=(element,),
    )

    assert is_element_in_viewport(session, element)

    response = element_send_keys(session, element, "cat")
    assert_success(response)

    assert is_element_in_viewport(session, element)
