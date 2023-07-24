from tests.support.asserts import assert_error, assert_is_active_element, assert_success


def read_global(session, name):
    return session.execute_script("return %s;" % name)


def get_active_element(session):
    return session.transport.send(
        "GET", "session/{session_id}/element/active".format(**vars(session)))


def test_no_top_browsing_context(session, closed_window):
    response = get_active_element(session)
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame):
    response = get_active_element(session)
    assert_error(response, "no such window")


def test_no_such_element(session, inline):
    session.url = inline("<body></body>")
    session.execute_script("""
        if (document.body.remove) {
          document.body.remove();
        } else {
          document.body.removeNode(true);
        }""")

    response = get_active_element(session)
    assert_error(response, "no such element")


def test_success_document(session, inline):
    session.url = inline("""
        <body>
            <h1>Heading</h1>
            <input />
            <input />
            <input style="opacity: 0" />
            <p>Another element</p>
        </body>""")

    response = get_active_element(session)
    element = assert_success(response)
    assert_is_active_element(session, element)


def test_success_input(session, inline):
    session.url = inline("""
        <body>
            <h1>Heading</h1>
            <input autofocus />
            <input style="opacity: 0" />
            <p>Another element</p>
        </body>""")

    # Per spec, autofocus candidates will be
    # flushed by next paint, so we use rAF here to
    # ensure the candidates are flushed.
    session.execute_async_script(
        """
        const resolve = arguments[0];
        window.requestAnimationFrame(function() {
            window.requestAnimationFrame(resolve);
        });
        """
    )
    response = get_active_element(session)
    element = assert_success(response)
    assert_is_active_element(session, element)


def test_success_input_non_interactable(session, inline):
    session.url = inline("""
        <body>
            <h1>Heading</h1>
            <input />
            <input style="opacity: 0" autofocus />
            <p>Another element</p>
        </body>""")

    # Per spec, autofocus candidates will be
    # flushed by next paint, so we use rAF here to
    # ensure the candidates are flushed.
    session.execute_async_script(
        """
        const resolve = arguments[0];
        window.requestAnimationFrame(function() {
            window.requestAnimationFrame(resolve);
        });
        """
    )
    response = get_active_element(session)
    element = assert_success(response)
    assert_is_active_element(session, element)


def test_success_explicit_focus(session, inline):
    session.url = inline("""
        <body>
            <h1>Heading</h1>
            <input />
            <iframe></iframe>
        </body>""")

    session.execute_script("document.body.getElementsByTagName('h1')[0].focus()")
    response = get_active_element(session)
    element = assert_success(response)
    assert_is_active_element(session, element)

    session.execute_script("document.body.getElementsByTagName('input')[0].focus()")
    response = get_active_element(session)
    element = assert_success(response)
    assert_is_active_element(session, element)

    session.execute_script("document.body.getElementsByTagName('iframe')[0].focus()")
    response = get_active_element(session)
    element = assert_success(response)
    assert_is_active_element(session, element)

    session.execute_script("document.body.getElementsByTagName('iframe')[0].focus();")
    session.execute_script("""
        var iframe = document.body.getElementsByTagName('iframe')[0];
        if (iframe.remove) {
          iframe.remove();
        } else {
          iframe.removeNode(true);
        }""")
    response = get_active_element(session)
    element = assert_success(response)
    assert_is_active_element(session, element)

    session.execute_script("document.body.appendChild(document.createElement('textarea'))")
    response = get_active_element(session)
    element = assert_success(response)
    assert_is_active_element(session, element)


def test_success_iframe_content(session, inline):
    session.url = inline("<body></body>")
    session.execute_script("""
        let iframe = document.createElement('iframe');
        document.body.appendChild(iframe);
        let input = iframe.contentDocument.createElement('input');
        iframe.contentDocument.body.appendChild(input);
        input.focus();
        """)

    response = get_active_element(session)
    element = assert_success(response)
    assert_is_active_element(session, element)
