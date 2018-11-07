from tests.support.asserts import assert_error, assert_success
from tests.support.inline import iframe, inline


def element_send_keys(session, element, text):
    return session.transport.send(
        "POST", "/session/{session_id}/element/{element_id}/value".format(
            session_id=session.session_id,
            element_id=element.id),
        {"text": text})


def test_body_is_interactable(session):
    session.url = inline("""
        <body onkeypress="document.querySelector('input').value += event.key">
          <input>
        </body>
    """)

    element = session.find.css("body", all=False)
    result = session.find.css("input", all=False)

    # By default body is the active element
    assert session.active_element == element

    response = element_send_keys(session, element, "foo")
    assert_success(response)
    assert session.active_element == element
    assert result.property("value") == "foo"


def test_document_element_is_interactable(session):
    session.url = inline("""
        <html onkeypress="document.querySelector('input').value += event.key">
          <input>
        </html>
    """)

    body = session.find.css("body", all=False)
    element = session.find.css(":root", all=False)
    result = session.find.css("input", all=False)

    # By default body is the active element
    assert session.active_element == body

    response = element_send_keys(session, element, "foo")
    assert_success(response)
    assert session.active_element == element
    assert result.property("value") == "foo"


def test_iframe_is_interactable(session):
    session.url = inline(iframe("""
        <body onkeypress="document.querySelector('input').value += event.key">
          <input>
        </body>
    """))

    body = session.find.css("body", all=False)
    frame = session.find.css("iframe", all=False)

    # By default the body has the focus
    assert session.active_element == body

    response = element_send_keys(session, frame, "foo")
    assert_success(response)
    assert session.active_element == frame

    # Any key events are immediately routed to the nested
    # browsing context's active document.
    session.switch_frame(frame)
    result = session.find.css("input", all=False)
    assert result.property("value") == "foo"


def test_transparent_element(session):
    session.url = inline("""<input style="opacity: 0">""")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, "foo")
    assert_success(response)
    assert element.property("value") == "foo"


def test_readonly_element(session):
    session.url = inline("<input readonly>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, "foo")
    assert_success(response)
    assert element.property("value") == ""


def test_obscured_element(session):
    session.url = inline("""
      <input>
      <div style="position: relative; top: -3em; height: 5em; background: blue;"></div>
    """)
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, "foo")
    assert_success(response)
    assert element.property("value") == "foo"


def test_not_a_focusable_element(session):
    session.url = inline("<div>foo</div>")
    element = session.find.css("div", all=False)

    response = element_send_keys(session, element, "foo")
    assert_error(response, "element not interactable")


def test_display_none(session):
    session.url = inline("""<input style="display: none">""")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, "foo")
    assert_error(response, "element not interactable")


def test_visibility_hidden(session):
    session.url = inline("""<input style="visibility: hidden">""")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, "foo")
    assert_error(response, "element not interactable")


def test_hidden(session):
    session.url = inline("<input hidden>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, "foo")
    assert_error(response, "element not interactable")


def test_disabled(session):
    session.url = inline("""<input disabled>""")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, "foo")
    assert_error(response, "element not interactable")
