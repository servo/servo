import pytest

from tests.support.asserts import assert_error, assert_success
from tests.support.dom import BUTTON_TYPES, INPUT_TYPES
from . import element_clear


@pytest.mark.parametrize("type", BUTTON_TYPES)
def test_button(session, inline, type):
    session.url = inline(f"""<button type="{type}" disabled>""")
    element = session.find.css("button", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


@pytest.mark.parametrize("type", INPUT_TYPES)
def test_input(session, inline, type):
    session.url = inline(f"""<input type="{type}" disabled>""")
    element = session.find.css("input", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_textarea(session, inline):
    session.url = inline("<textarea disabled></textarea>")
    element = session.find.css("textarea", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_fieldset_descendant(session, inline):
    session.url = inline("<fieldset disabled><input>foo")
    element = session.find.css("input", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_fieldset_descendant_first_legend(session, inline):
    session.url = inline("<fieldset disabled><legend><input>foo")
    element = session.find.css("input", all=False)

    response = element_clear(session, element)
    assert_success(response)


def test_fieldset_descendant_not_first_legend(session, inline):
    session.url = inline("<fieldset disabled><legend></legend><legend><input>foo")
    element = session.find.css("input", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_option(session, inline):
    session.url = inline("<select><option disabled>foo")
    element = session.find.css("option", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_option_optgroup(session, inline):
    session.url = inline("<select><optgroup disabled><option>foo")
    element = session.find.css("option", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_option_select(session, inline):
    session.url = inline("<select disabled><option>foo")
    element = session.find.css("option", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_optgroup_select(session, inline):
    session.url = inline("<select disabled><optgroup>foo")
    element = session.find.css("optgroup", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


def test_select(session, inline):
    session.url = inline("<select disabled>")
    element = session.find.css("select", all=False)

    response = element_clear(session, element)
    assert_error(response, "invalid element state")


@pytest.mark.parametrize("tagname", ["button", "input", "select", "textarea"])
def test_xhtml(session, inline, tagname):
    session.url = inline(
        f"""<{tagname} disabled="disabled"></{tagname}>""", doctype="xhtml")
    element = session.find.css(tagname, all=False)

    result = element_clear(session, element)
    assert_error(result, "invalid element state")


def test_xml(session, inline):
    session.url = inline("""<note></note>""", doctype="xml")
    element = session.find.css("note", all=False)

    result = element_clear(session, element)
    assert_error(result, "invalid element state")
