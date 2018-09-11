import pytest

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def is_element_enabled(session, element_id):
    return session.transport.send(
        "GET",
        "session/{session_id}/element/{element_id}/enabled".format(
            session_id=session.session_id,
            element_id=element_id
        )
    )


def test_no_browsing_context(session, closed_window):
    response = is_element_enabled(session, "foo")
    assert_error(response, "no such window")


def test_element_stale(session):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)
    session.refresh()

    result = is_element_enabled(session, element.id)
    assert_error(result, "stale element reference")


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_form_control_disabled(session, element):
    session.url = inline("<{} disabled/>".format(element))
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, False)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_form_control_enabled(session, element):
    session.url = inline("<{}/>".format(element))
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, True)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_fieldset_disabled_descendant(session, element):
    session.url = inline("<fieldset disabled><{}/></fieldset>".format(element))
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, False)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_fieldset_enabled_descendant(session, element):
    session.url = inline("<fieldset><{}/></fieldset>".format(element))
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, True)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_fieldset_disabled_descendant_legend(session, element):
    session.url = inline("<fieldset disabled><legend><{}/></legend></fieldset>".format(element))
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, True)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_fieldset_enabled_descendant_legend(session, element):
    session.url = inline("<fieldset><legend><{}/></legend></fieldset>".format(element))
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, True)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_xhtml_form_control_disabled(session, element):
    session.url = inline("""<{} disabled="disabled"/>""".format(element),
                         doctype="xhtml")
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, False)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_xhtml_form_control_enabled(session, element):
    session.url = inline("""<{}/>""".format(element), doctype="xhtml")
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, True)


def test_xml_always_not_enabled(session):
    session.url = inline("""<note></note>""", doctype="xml")
    element = session.find.css("note", all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, False)
