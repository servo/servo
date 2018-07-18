from webdriver import Element, NoSuchAlertException, WebDriverException


# WebDriver specification ID: dfn-error-response-data
errors = {
    "element click intercepted": 400,
    "element not selectable": 400,
    "element not interactable": 400,
    "insecure certificate": 400,
    "invalid argument": 400,
    "invalid cookie domain": 400,
    "invalid coordinates": 400,
    "invalid element state": 400,
    "invalid selector": 400,
    "invalid session id": 404,
    "javascript error": 500,
    "move target out of bounds": 500,
    "no such alert": 404,
    "no such cookie": 404,
    "no such element": 404,
    "no such frame": 404,
    "no such window": 404,
    "script timeout": 408,
    "session not created": 500,
    "stale element reference": 404,
    "timeout": 408,
    "unable to set cookie": 500,
    "unable to capture screen": 500,
    "unexpected alert open": 500,
    "unknown command": 404,
    "unknown error": 500,
    "unknown method": 405,
    "unsupported operation": 500,
}


# WebDriver specification ID: dfn-send-an-error
#
# > When required to send an error, with error code, a remote end must run the
# > following steps:
# >
# > 1. Let http status and name be the error response data for error code.
# > 2. Let message be an implementation-defined string containing a
# >    human-readable description of the reason for the error.
# > 3. Let stacktrace be an implementation-defined string containing a stack
# >    trace report of the active stack frames at the time when the error
# >    occurred.
# > 4. Let data be a new JSON Object initialised with the following properties:
# >
# >     error
# >         name
# >     message
# >         message
# >     stacktrace
# >         stacktrace
# >
# > 5. Send a response with status and data as arguments.
def assert_error(response, error_code):
    """
    Verify that the provided webdriver.Response instance described
    a valid error response as defined by `dfn-send-an-error` and
    the provided error code.

    :param response: ``webdriver.Response`` instance.
    :param error_code: String value of the expected error code
    """
    assert response.status == errors[error_code]
    assert "value" in response.body
    assert response.body["value"]["error"] == error_code
    assert isinstance(response.body["value"]["message"], basestring)
    assert isinstance(response.body["value"]["stacktrace"], basestring)


def assert_success(response, value=None):
    """
    Verify that the provided webdriver.Response instance described
    a valid error response as defined by `dfn-send-an-error` and
    the provided error code.

    :param response: ``webdriver.Response`` instance.
    :param value: Expected value of the response body, if any.
    """
    assert response.status == 200, str(response.error)

    if value is not None:
        assert response.body["value"] == value
    return response.body.get("value")


def assert_dialog_handled(session, expected_text):
    # If there were any existing dialogs prior to the creation of this
    # fixture's dialog, then the "Get Alert Text" command will return
    # successfully. In that case, the text must be different than that
    # of this fixture's dialog.
    try:
        assert session.alert.text != expected_text, (
            "User prompt with text '%s' was not handled." % expected_text)

    except NoSuchAlertException:
        pass


def assert_files_uploaded(session, element, files):

    def get_file_contents(file_index):
        return session.execute_async_script("""
            let files = arguments[0].files;
            let index = arguments[1];
            let resolve = arguments[2];

            var reader = new FileReader();
            reader.onload = function(event) {
              resolve(reader.result);
            };
            reader.readAsText(files[index]);
        """, (element, file_index))

    def get_uploaded_file_names():
        return session.execute_script("""
            let fileList = arguments[0].files;
            let files = [];

            for (var i = 0; i < fileList.length; i++) {
              files.push(fileList[i].name);
            }

            return files;
        """, args=(element,))

    expected_file_names = [str(f.basename) for f in files]
    assert get_uploaded_file_names() == expected_file_names

    for index, f in enumerate(files):
        assert get_file_contents(index) == f.read()


def assert_same_element(session, a, b):
    """Verify that two element references describe the same element."""
    if isinstance(a, dict):
        assert Element.identifier in a, "Actual value does not describe an element"
        a_id = a[Element.identifier]
    elif isinstance(a, Element):
        a_id = a.id
    else:
        raise AssertionError("Actual value is not a dictionary or web element")

    if isinstance(b, dict):
        assert Element.identifier in b, "Expected value does not describe an element"
        b_id = b[Element.identifier]
    elif isinstance(b, Element):
        b_id = b.id
    else:
        raise AssertionError("Expected value is not a dictionary or web element")

    if a_id == b_id:
        return

    message = ("Expected element references to describe the same element, " +
               "but they did not.")

    # Attempt to provide more information, accounting for possible errors such
    # as stale element references or not visible elements.
    try:
        a_markup = session.execute_script("return arguments[0].outerHTML;", args=(a,))
        b_markup = session.execute_script("return arguments[0].outerHTML;", args=(b,))
        message += " Actual: `%s`. Expected: `%s`." % (a_markup, b_markup)
    except WebDriverException:
        pass

    raise AssertionError(message)


def assert_in_events(session, expected_events):
    actual_events = session.execute_script("return window.events")
    for expected_event in expected_events:
        assert expected_event in actual_events


def assert_events_equal(session, expected_events):
    actual_events = session.execute_script("return window.events")
    assert actual_events == expected_events


def assert_element_has_focus(target_element):
    session = target_element.session

    active_element = session.execute_script("return document.activeElement")
    active_tag = active_element.property("localName")
    target_tag = target_element.property("localName")

    assert active_element == target_element, (
        "Focussed element is <%s>, not <%s>" % (active_tag, target_tag))


def assert_move_to_coordinates(point, target, events):
    for e in events:
        if e["type"] != "mousemove":
            assert e["pageX"] == point["x"]
            assert e["pageY"] == point["y"]
            assert e["target"] == target
