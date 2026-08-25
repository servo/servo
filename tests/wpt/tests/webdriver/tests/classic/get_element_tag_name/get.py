import pytest

from webdriver import WebElement

from tests.support.classic.asserts import assert_error, assert_success


HTML_NAMESPACE = "http://www.w3.org/1999/xhtml"
SVG_NAMESPACE = "http://www.w3.org/2000/svg"


def get_element_tag_name(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/name".format(
            session_id=session.session_id,
            element_id=element_id))


def test_no_top_browsing_context(session, closed_window):
    original_handle, element = closed_window
    response = get_element_tag_name(session, element.id)
    assert_error(response, "no such window")
    response = get_element_tag_name(session, "foo")
    assert_error(response, "no such window")

    session.window_handle = original_handle
    response = get_element_tag_name(session, element.id)
    assert_error(response, "no such element")


def test_no_browsing_context(session, closed_frame):
    response = get_element_tag_name(session, "foo")
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = WebElement(session, "foo")

    response = get_element_tag_name(session, element.id)
    assert_error(response, "no such element")


def test_no_such_element_with_shadow_root(session, get_test_page):
    session.url = get_test_page()

    element = session.find.css("custom-element", all=False)

    result = get_element_tag_name(session, element.shadow_root.id)
    assert_error(result, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_window_handle(session, inline, closed):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()

    if closed:
        session.window.close()

    session.window_handle = new_handle

    response = get_element_tag_name(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_frame(session, get_test_page, closed):
    session.url = get_test_page(as_frame=True)

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    element = session.find.css("div", all=False)

    session.switch_to_parent_frame()

    if closed:
        session.execute_script("arguments[0].remove();", args=[frame])

    response = get_element_tag_name(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("input#text", as_frame=as_frame)

    result = get_element_tag_name(session, element.id)
    assert_error(result, "stale element reference")


@pytest.mark.parametrize(
    "markup",
    ["<input id=foo>", "<INPUT id=foo>"],
    ids=["lowercase", "uppercase"],
)
def test_get_element_tag_name(session, inline, markup):
    session.url = inline(markup)
    element = session.find.css("input", all=False)

    result = get_element_tag_name(session, element.id)
    assert_success(result, "input")


def test_get_svg_element_tag_name(session, inline):
    session.url = inline("<svg><lineargradient id=foo></lineargradient></svg>")
    element = session.find.css("#foo", all=False)

    result = get_element_tag_name(session, element.id)
    assert_success(result, "linearGradient")


def test_get_element_tag_name_with_namespace_prefix(session, inline):
    session.url = inline(
        f"""<root xmlns:SvG="{SVG_NAMESPACE}"><SvG:linearGradient/></root>""",
        doctype="xml")
    element = session.find.css("linearGradient", all=False)

    result = get_element_tag_name(session, element.id)
    assert_success(result, "SvG:linearGradient")


def test_get_element_tag_name_xhtml(session, inline):
    session.url = inline("<div></div>", doctype="xhtml")
    element = session.find.css("div", all=False)
    result = get_element_tag_name(session, element.id)
    assert_success(result, "div")


@pytest.mark.parametrize(
    "namespace, local_name, expected",
    [
        (HTML_NAMESPACE, "I", "I"),
        (HTML_NAMESPACE, "i", "i"),
        (HTML_NAMESPACE, "x:b", "x:b"),
        (SVG_NAMESPACE, "svg", "svg"),
        (SVG_NAMESPACE, "SVG", "SVG"),
        (SVG_NAMESPACE, "s:svg", "s:svg"),
        (SVG_NAMESPACE, "s:SVG", "s:SVG"),
        (SVG_NAMESPACE, "textPath", "textPath"),
        ("http://example.com/", "mixedCase", "mixedCase"),
    ],
    ids=[
        "html upper",
        "html lower",
        "html prefixed",
        "svg lower",
        "svg upper",
        "svg lower prefixed",
        "svg upper prefixed",
        "svg mixed",
        "custom mixed",
    ],
)
def test_get_element_tag_name_namespace_created_via_script(
    session, inline, namespace, local_name, expected
):
    session.url = inline("")
    element = session.execute_script(
        """
        var el = document.createElementNS(arguments[0], arguments[1]);
        document.body.appendChild(el);
        return el;
        """,
        args=[namespace, local_name],
    )

    result = get_element_tag_name(session, element.id)
    assert_success(result, expected)
