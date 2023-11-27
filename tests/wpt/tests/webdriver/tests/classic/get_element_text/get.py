import pytest

from webdriver import WebElement

from tests.support.asserts import assert_error, assert_success


def get_element_text(session, element_id):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/text".format(
            session_id=session.session_id,
            element_id=element_id))


def test_no_top_browsing_context(session, closed_window):
    original_handle, element = closed_window
    response = get_element_text(session, element.id)
    assert_error(response, "no such window")
    response = get_element_text(session, "foo")
    assert_error(response, "no such window")

    session.window_handle = original_handle
    response = get_element_text(session, element.id)
    assert_error(response, "no such element")


def test_no_browsing_context(session, closed_frame):
    response = get_element_text(session, "foo")
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = WebElement(session, "foo")

    response = get_element_text(session, element.id)
    assert_error(response, "no such element")


def test_no_such_element_with_shadow_root(session, get_test_page):
    session.url = get_test_page()

    element = session.find.css("custom-element", all=False)

    result = get_element_text(session, element.shadow_root.id)
    assert_error(result, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_window_handle(session, inline, closed):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()

    if closed:
        session.window.close()

    session.window_handle = new_handle

    response = get_element_text(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_frame(session, get_test_page, closed):
    session.url = get_test_page(as_frame=True)

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)

    element = session.find.css("div", all=False)

    session.switch_frame("parent")

    if closed:
        session.execute_script("arguments[0].remove();", args=[frame])

    response = get_element_text(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("input#text", as_frame=as_frame)

    response = get_element_text(session, element.id)
    assert_error(response, "stale element reference")


def test_getting_text_of_a_non_existant_element_is_an_error(session, inline):
    session.url = inline("""<body>Hello world</body>""")

    result = get_element_text(session, "foo")
    assert_error(result, "no such element")


def test_read_element_text(session, inline):
    session.url = inline("Before f<span id='id'>oo</span> after")
    element = session.find.css("#id", all=False)

    result = get_element_text(session, element.id)
    assert_success(result, "oo")


@pytest.mark.parametrize("text, inner_html, expected", [
    ("cheese", "<slot><span>foo</span>bar</slot>", "cheese"),
    ("cheese", "<slot><span>foo</span></slot>bar", "cheesebar"),
    ("cheese", "<slot><span style=\"display: none\">foo</span>bar</slot>", "cheese"),
    ("", "<slot><span>foo</span>bar</slot>", "foobar"),
    ("", "<slot><span>foo</span></slot>bar", "foobar"),
    ("", "<slot><span style=\"display: none\">foo</span>bar</slot>", "bar"),
], ids=[
    "custom visible",
    "custom outside",
    "custom hidden",
    "default visible",
    "default outside",
    "default hidden",
])
def test_shadow_root_slot(session, inline, text, inner_html, expected):
    session.url = inline(f"""
        <test-container>{text}</test-container>
        <script>
            class TestContainer extends HTMLElement {{
                connectedCallback() {{
                    const shadow = this.attachShadow({{ mode: "open" }});
                    shadow.innerHTML = "{inner_html}";
                }}
            }}

            customElements.define("test-container", TestContainer);
        </script>
        """)

    element = session.find.css("test-container", all=False)

    result = get_element_text(session, element.id)
    assert_success(result, expected)


def test_pretty_print_xml(session, inline):
    session.url = inline("<xml><foo>che<bar>ese</bar></foo></xml>", doctype="xml")

    elem = session.find.css("foo", all=False)
    assert elem.text == "cheese"
