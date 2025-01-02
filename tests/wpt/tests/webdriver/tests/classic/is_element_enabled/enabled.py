import pytest

from webdriver import WebElement

from tests.support.asserts import assert_error, assert_success
from tests.support.dom import BUTTON_TYPES, INPUT_TYPES
from . import is_element_enabled


def test_no_top_browsing_context(session, closed_window):
    original_handle, element = closed_window
    response = is_element_enabled(session, element.id)
    assert_error(response, "no such window")
    response = is_element_enabled(session, "foo")
    assert_error(response, "no such window")

    session.window_handle = original_handle
    response = is_element_enabled(session, element.id)
    assert_error(response, "no such element")


def test_no_browsing_context(session, closed_frame):
    response = is_element_enabled(session, "foo")
    assert_error(response, "no such window")


def test_no_such_element_with_invalid_value(session):
    element = WebElement(session, "foo")

    response = is_element_enabled(session, element.id)
    assert_error(response, "no such element")


def test_no_such_element_with_shadow_root(session, get_test_page):
    session.url = get_test_page()

    element = session.find.css("custom-element", all=False)

    result = is_element_enabled(session, element.shadow_root.id)
    assert_error(result, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_window_handle(session, inline, closed):
    session.url = inline("<div id='parent'><p/>")
    element = session.find.css("#parent", all=False)

    new_handle = session.new_window()

    if closed:
        session.window.close()

    session.window_handle = new_handle

    response = is_element_enabled(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("closed", [False, True], ids=["open", "closed"])
def test_no_such_element_from_other_frame(session, get_test_page, closed):
    session.url = get_test_page(as_frame=True)

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)

    element = session.find.css("input#text", all=False)

    session.switch_frame("parent")

    if closed:
        session.execute_script("arguments[0].remove();", args=[frame])

    response = is_element_enabled(session, element.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("input#text", as_frame=as_frame)

    result = is_element_enabled(session, element.id)
    assert_error(result, "stale element reference")


@pytest.mark.parametrize("status, expected", [
    ("", True),
    ("disabled", False),
], ids=["enabled", "disabled"])
@pytest.mark.parametrize("type", BUTTON_TYPES)
def test_button(session, inline, status, expected, type):
    session.url = inline(f"""<button type="{type}" {status}>""")
    element = session.find.css("button", all=False)

    response = is_element_enabled(session, element.id)
    assert_success(response, expected)


@pytest.mark.parametrize("status, expected", [
    ("", True),
    ("disabled", False),
], ids=["enabled", "disabled"])
@pytest.mark.parametrize("type", INPUT_TYPES)
def test_input(session, inline, status, expected, type):
    session.url = inline(f"""<input type="{type}" {status}>""")
    element = session.find.css("input", all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, expected)


@pytest.mark.parametrize("status, expected", [
    ("", True),
    ("disabled", False),
], ids=["enabled", "disabled"])
def test_textarea(session, inline, status, expected):
    session.url = inline(f"<textarea {status}></textarea>")
    element = session.find.css("textarea", all=False)

    response = is_element_enabled(session, element.id)
    assert_success(response, expected)


@pytest.mark.parametrize("status, expected", [
    ("", True),
    ("disabled", False),
], ids=["enabled", "disabled"])
def test_fieldset(session, inline, status, expected):
    session.url = inline(f"<fieldset {status}><input>foo")
    element = session.find.css("input", all=False)

    response = is_element_enabled(session, element.id)
    assert_success(response, expected)


@pytest.mark.parametrize("status, expected", [
    ("", True),
    ("disabled", False),
], ids=["enabled", "disabled"])
def test_fieldset_descendant(session, inline, status, expected):
    session.url = inline(f"<fieldset {status}><input>foo")
    element = session.find.css("input", all=False)

    response = is_element_enabled(session, element.id)
    assert_success(response, expected)


@pytest.mark.parametrize("status, expected", [
    ("", True),
    ("disabled", True),
], ids=["enabled", "disabled"])
def test_fieldset_descendant_first_legend(session, inline, status, expected):
    session.url = inline(f"<fieldset {status}><legend><input>foo")
    element = session.find.css("input", all=False)

    response = is_element_enabled(session, element.id)
    assert_success(response, expected)


@pytest.mark.parametrize("status, expected", [
    ("", True),
    ("disabled", False),
], ids=["enabled", "disabled"])
def test_fieldset_descendant_not_first_legend(session, inline, status, expected):
    session.url = inline(f"<fieldset {status}><legend></legend><legend><input>foo")
    element = session.find.css("input", all=False)

    response = is_element_enabled(session, element.id)
    assert_success(response, expected)


@pytest.mark.parametrize("status, expected", [
    ("", True),
    ("disabled", False),
], ids=["enabled", "disabled"])
def test_option(session, inline, status, expected):
    session.url = inline(f"<select><option {status}>foo")
    element = session.find.css("option", all=False)

    response = is_element_enabled(session, element.id)
    assert_success(response, expected)


@pytest.mark.parametrize("status, expected", [
    ("", True),
    ("disabled", False),
], ids=["enabled", "disabled"])
def test_option_with_optgroup(session, inline, status, expected):
    session.url = inline(f"<select><optgroup {status}><option>foo")
    element = session.find.css("optgroup", all=False)

    response = is_element_enabled(session, element.id)
    assert_success(response, expected)

    option = session.find.css("option", all=False)
    response = is_element_enabled(session, option.id)
    assert_success(response, expected)


@pytest.mark.parametrize("status, expected", [
    ("", True),
    ("disabled", False),
], ids=["enabled", "disabled"])
def test_option_with_select(session, inline, status, expected):
    session.url = inline(f"<select {status}><option>foo")

    option = session.find.css("option", all=False)
    response = is_element_enabled(session, option.id)
    assert_success(response, expected)


@pytest.mark.parametrize("status, expected", [
    ("", True),
    ("disabled", False),
], ids=["enabled", "disabled"])
def test_optgroup_with_select(session, inline, status, expected):
    session.url = inline(f"<select {status}><optgroup>foo")

    option = session.find.css("optgroup", all=False)
    response = is_element_enabled(session, option.id)
    assert_success(response, expected)


@pytest.mark.parametrize("status, expected", [
    ("", True),
    ("disabled", False),
], ids=["enabled", "disabled"])
def test_select(session, inline, status, expected):
    session.url = inline(f"<select {status}>")
    element = session.find.css("select", all=False)

    response = is_element_enabled(session, element.id)
    assert_success(response, expected)


@pytest.mark.parametrize("status, expected", [
    ("", True),
    ("disabled=\"disabled\"", False),
], ids=["enabled", "disabled"])
@pytest.mark.parametrize("tagname", ["button", "input", "select", "textarea"])
def test_xhtml(session, inline, status, expected, tagname):
    session.url = inline(
        f"""<{tagname} {status}></{tagname}>""", doctype="xhtml")
    element = session.find.css(tagname, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, expected)


def test_xml(session, inline):
    session.url = inline("""<note></note>""", doctype="xml")
    element = session.find.css("note", all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, False)
