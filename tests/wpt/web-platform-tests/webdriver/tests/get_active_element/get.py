from tests.support.asserts import assert_error, assert_same_element
from tests.support.inline import inline


def read_global(session, name):
    return session.execute_script("return %s;" % name)


def get_active_element(session):
    return session.transport.send(
        "GET", "session/{session_id}/element/active".format(**vars(session)))


def assert_is_active_element(session, response):
    """Ensure that the provided object is a successful WebDriver
    response describing an element reference and that the referenced
    element matches the element returned by the `activeElement`
    attribute of the current browsing context's active document.

    """
    assert response.status == 200
    assert "value" in response.body

    from_js = session.execute_script("return document.activeElement")

    if response.body["value"] is None:
        assert from_js is None
    else:
        assert_same_element(session, response.body["value"], from_js)


def test_closed_context(session, create_window):
    """
    > 1. If the current browsing context is no longer open, return error with
    >    error code no such window.
    """
    new_window = create_window()
    session.window_handle = new_window
    session.close()

    response = get_active_element(session)
    assert_error(response, "no such window")


def test_success_document(session):
    """
    > [...]
    > 3. Let active element be the active element of the current browsing
    >    context's document element.
    > 4. Let active web element be the JSON Serialization of active element.
    > 5. Return success with data active web element.
    """
    session.url = inline("""
        <body>
            <h1>Heading</h1>
            <input />
            <input />
            <input style="opacity: 0" />
            <p>Another element</p>
        </body>""")
    response = get_active_element(session)
    assert_is_active_element(session, response)


def test_sucess_input(session):
    session.url = inline("""
        <body>
            <h1>Heading</h1>
            <input autofocus />
            <input style="opacity: 0" />
            <p>Another element</p>
        </body>""")
    response = get_active_element(session)
    assert_is_active_element(session, response)


def test_sucess_input_non_interactable(session):
    session.url = inline("""
        <body>
            <h1>Heading</h1>
            <input />
            <input style="opacity: 0" autofocus />
            <p>Another element</p>
        </body>""")
    response = get_active_element(session)
    assert_is_active_element(session, response)


def test_success_explicit_focus(session):
    session.url = inline("""
        <body>
            <h1>Heading</h1>
            <input />
            <iframe></iframe>
        </body>""")

    session.execute_script("document.body.getElementsByTagName('h1')[0].focus()")
    response = get_active_element(session)
    assert_is_active_element(session, response)

    session.execute_script("document.body.getElementsByTagName('input')[0].focus()")
    response = get_active_element(session)
    assert_is_active_element(session, response)

    session.execute_script("document.body.getElementsByTagName('iframe')[0].focus()")
    response = get_active_element(session)
    assert_is_active_element(session, response)

    session.execute_script("document.body.getElementsByTagName('iframe')[0].focus();")
    session.execute_script("""
        var iframe = document.body.getElementsByTagName('iframe')[0];
        if (iframe.remove) {
          iframe.remove();
        } else {
          iframe.removeNode(true);
        }""")
    response = get_active_element(session)
    assert_is_active_element(session, response)

    session.execute_script("document.body.appendChild(document.createElement('textarea'))")
    response = get_active_element(session)
    assert_is_active_element(session, response)


def test_success_iframe_content(session):
    session.url = inline("<body></body>")
    session.execute_script("""
        let iframe = document.createElement('iframe');
        document.body.appendChild(iframe);
        let input = iframe.contentDocument.createElement('input');
        iframe.contentDocument.body.appendChild(input);
        input.focus();
        """)

    response = get_active_element(session)
    assert_is_active_element(session, response)


def test_missing_document_element(session):
    session.url = inline("<body></body>")
    session.execute_script("""
        if (document.body.remove) {
          document.body.remove();
        } else {
          document.body.removeNode(true);
        }""")

    response = get_active_element(session)
    assert_error(response, "no such element")
