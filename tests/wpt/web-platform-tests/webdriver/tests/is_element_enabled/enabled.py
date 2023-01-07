import pytest

from webdriver import Element

from tests.support.asserts import assert_error, assert_success


def is_element_enabled(session, element_id):
    return session.transport.send(
        "GET",
        "session/{session_id}/element/{element_id}/enabled".format(
            session_id=session.session_id,
            element_id=element_id
        )
    )


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
    element = Element("foo", session)

    response = is_element_enabled(session, element.id)
    assert_error(response, "no such element")


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
def test_no_such_element_from_other_frame(session, url, closed):
    session.url = url("/webdriver/tests/support/html/subframe.html")

    frame = session.find.css("#delete-frame", all=False)
    session.switch_frame(frame)

    button = session.find.css("#remove-parent", all=False)
    if closed:
        button.click()

    session.switch_frame("parent")

    response = is_element_enabled(session, button.id)
    assert_error(response, "no such element")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("<input>", "input", as_frame=as_frame)

    result = is_element_enabled(session, element.id)
    assert_error(result, "stale element reference")


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_form_control_disabled(session, inline, element):
    session.url = inline("<{} disabled/>".format(element))
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, False)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_form_control_enabled(session, inline, element):
    session.url = inline("<{}/>".format(element))
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, True)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_fieldset_disabled_descendant(session, inline, element):
    session.url = inline("<fieldset disabled><{}/></fieldset>".format(element))
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, False)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_fieldset_enabled_descendant(session, inline, element):
    session.url = inline("<fieldset><{}/></fieldset>".format(element))
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, True)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_fieldset_disabled_descendant_legend(session, inline, element):
    session.url = inline("<fieldset disabled><legend><{}/></legend></fieldset>".format(element))
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, True)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_fieldset_enabled_descendant_legend(session, inline, element):
    session.url = inline("<fieldset><legend><{}/></legend></fieldset>".format(element))
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, True)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_xhtml_form_control_disabled(session, inline, element):
    session.url = inline("""<{} disabled="disabled"/>""".format(element),
                         doctype="xhtml")
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, False)


@pytest.mark.parametrize("element", ["button", "input", "select", "textarea"])
def test_xhtml_form_control_enabled(session, inline, element):
    session.url = inline("""<{}/>""".format(element), doctype="xhtml")
    element = session.find.css(element, all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, True)


def test_xml_always_not_enabled(session, inline):
    session.url = inline("""<note></note>""", doctype="xml")
    element = session.find.css("note", all=False)

    result = is_element_enabled(session, element.id)
    assert_success(result, False)
