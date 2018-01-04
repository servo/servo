from tests.support.asserts import assert_error, assert_dialog_handled, assert_same_element
from tests.support.fixtures import create_dialog
from tests.support.inline import inline


def read_global(session, name):
    return session.execute_script("return %s;" % name)


def get_active_element(session):
    return session.transport.send("GET", "session/%s/element/active" % session.session_id)


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


# > 1. If the current browsing context is no longer open, return error with
# >    error code no such window.
def test_closed_context(session, create_window):
    new_window = create_window()
    session.window_handle = new_window
    session.close()

    response = get_active_element(session)
    assert_error(response, "no such window")


# [...]
# 2. Handle any user prompts and return its value if it is an error.
# [...]
# In order to handle any user prompts a remote end must take the following
# steps:
# 2. Run the substeps of the first matching user prompt handler:
#
#    [...]
#    - dismiss state
#      1. Dismiss the current user prompt.
#    [...]
#
# 3. Return success.
def test_handle_prompt_dismiss(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "dismiss"})}})
    session.url = inline("<body><p>Hello, World!</p></body>")

    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")

    response = get_active_element(session)
    assert_is_active_element(session, response)
    assert_dialog_handled(session, "dismiss #1")
    assert session.execute_script("return dismiss1") is None

    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")

    response = get_active_element(session)
    assert_is_active_element(session, response)
    assert_dialog_handled(session, "dismiss #2")
    assert read_global(session, "dismiss2") is False

    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")

    response = get_active_element(session)
    assert_is_active_element(session, response)
    assert_dialog_handled(session, "dismiss #3")
    assert read_global(session, "dismiss3") is None


# [...]
# 2. Handle any user prompts and return its value if it is an error.
# [...]
# In order to handle any user prompts a remote end must take the following
# steps:
# 2. Run the substeps of the first matching user prompt handler:
#
#    [...]
#    - accept state
#      1. Accept the current user prompt.
#    [...]
#
# 3. Return success.
def test_handle_prompt_accept(new_session, add_browser_capabilites):
    _, session = new_session({"capabilities": {"alwaysMatch": add_browser_capabilites({"unhandledPromptBehavior": "accept"})}})
    session.url = inline("<body><p>Hello, World!</p></body>")
    create_dialog(session)("alert", text="accept #1", result_var="accept1")

    response = get_active_element(session)
    assert_is_active_element(session, response)
    assert_dialog_handled(session, "accept #1")
    assert read_global(session, "accept1") is None

    create_dialog(session)("confirm", text="accept #2", result_var="accept2")

    response = get_active_element(session)
    assert_is_active_element(session, response)
    assert_dialog_handled(session, "accept #2")
    assert read_global(session, "accept2") is True

    create_dialog(session)("prompt", text="accept #3", result_var="accept3")

    response = get_active_element(session)
    assert_is_active_element(session, response)
    assert_dialog_handled(session, "accept #3")
    assert read_global(session, "accept3") == "" or read_global(session, "accept3") == "undefined"


# [...]
# 2. Handle any user prompts and return its value if it is an error.
# [...]
# In order to handle any user prompts a remote end must take the following
# steps:
# 2. Run the substeps of the first matching user prompt handler:
#
#    [...]
#    - missing value default state
#    - not in the table of simple dialogs
#      1. Dismiss the current user prompt.
#      2. Return error with error code unexpected alert open.
def test_handle_prompt_missing_value(session, create_dialog):
    session.url = inline("<body><p>Hello, World!</p></body>")

    create_dialog("alert", text="dismiss #1", result_var="dismiss1")

    response = get_active_element(session)
    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #1")
    assert session.execute_script("return dismiss1") is None

    create_dialog("confirm", text="dismiss #2", result_var="dismiss2")

    response = get_active_element(session)
    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #2")
    assert session.execute_script("return dismiss2") is False

    create_dialog("prompt", text="dismiss #3", result_var="dismiss3")

    response = get_active_element(session)
    assert_error(response, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #3")
    assert session.execute_script("return dismiss3") is None


# > [...]
# > 3. Let active element be the active element of the current browsing
# >    context's document element.
# > 4. Let active web element be the JSON Serialization of active element.
# > 5. Return success with data active web element.
def test_success_document(session):
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
