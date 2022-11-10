from tests.support.asserts import assert_element_has_focus


def element_send_keys(session, element, text):
    return session.transport.send(
        "POST", "/session/{session_id}/element/{element_id}/value".format(
            session_id=session.session_id,
            element_id=element.id),
        {"text": text})


def test_input(session, inline):
    session.url = inline("<input>")
    element = session.find.css("input", all=False)
    assert element.property("value") == ""

    element_send_keys(session, element, "foo")
    assert element.property("value") == "foo"
    assert_element_has_focus(element)


def test_textarea(session, inline):
    session.url = inline("<textarea>")
    element = session.find.css("textarea", all=False)
    assert element.property("value") == ""

    element_send_keys(session, element, "foo")
    assert element.property("value") == "foo"
    assert_element_has_focus(element)


def test_input_append(session, inline):
    session.url = inline("<input value=a>")
    body = session.find.css("body", all=False)
    assert_element_has_focus(body)
    element = session.find.css("input", all=False)
    assert element.property("value") == "a"

    element_send_keys(session, element, "b")
    assert_element_has_focus(element)
    assert element.property("value") == "ab"

    element_send_keys(session, element, "c")
    assert element.property("value") == "abc"


def test_textarea_append(session, inline):
    session.url = inline("<textarea>a</textarea>")
    body = session.find.css("body", all=False)
    assert_element_has_focus(body)
    element = session.find.css("textarea", all=False)
    assert element.property("value") == "a"

    element_send_keys(session, element, "b")
    assert_element_has_focus(element)
    assert element.property("value") == "ab"

    element_send_keys(session, element, "c")
    assert element.property("value") == "abc"


def test_input_insert_when_focused(session, inline):
    session.url = inline("""<input value=a>
<script>
let elem = document.getElementsByTagName("input")[0];
elem.focus();
elem.setSelectionRange(0, 0);
</script>""")
    element = session.find.css("input", all=False)
    assert element.property("value") == "a"

    element_send_keys(session, element, "b")
    assert element.property("value") == "ba"

    element_send_keys(session, element, "c")
    assert element.property("value") == "bca"


def test_textarea_insert_when_focused(session, inline):
    session.url = inline("""<textarea>a</textarea>
<script>
let elem = document.getElementsByTagName("textarea")[0];
elem.focus();
elem.setSelectionRange(0, 0);
</script>""")
    element = session.find.css("textarea", all=False)
    assert element.property("value") == "a"

    element_send_keys(session, element, "b")
    assert element.property("value") == "ba"

    element_send_keys(session, element, "c")
    assert element.property("value") == "bca"


def test_date(session, inline):
    session.url = inline("<input type=date>")
    element = session.find.css("input", all=False)

    element_send_keys(session, element, "2000-01-01")
    assert element.property("value") == "2000-01-01"
    assert_element_has_focus(element)
