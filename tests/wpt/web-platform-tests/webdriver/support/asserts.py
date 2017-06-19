from webdriver.client import Element, element_key
from webdriver.error import WebDriverException

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
    "no such alert": 400,
    "no such cookie": 404,
    "no such element": 404,
    "no such frame": 400,
    "no such window": 400,
    "script timeout": 408,
    "session not created": 500,
    "stale element reference": 400,
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
    """Verify that the provided wdclient.Response instance described a valid
    error response as defined by `dfn-send-an-error` and the provided error
    code.

    :param response: wdclient.Response instance
    :param error_code: string value of the expected "error code"
    """
    assert response.status == errors[error_code]
    assert "value" in response.body
    assert response.body["value"]["error"] == error_code
    assert isinstance(response.body["value"]["message"], basestring)
    assert isinstance(response.body["value"]["stacktrace"], basestring)

def assert_success(response, value):
    """Verify that the provided wdclient.Response instance described a valid
    error response as defined by `dfn-send-an-error` and the provided error
    code.
    :param response: wdclient.Response instance
    :param value: expected value of the response body
    """

    assert response.status == 200
    assert response.body["value"] == value

def assert_dialog_handled(session, expected_text):
    result = session.transport.send("GET",
                                    "session/%s/alert/text" % session.session_id)

    # If there were any existing dialogs prior to the creation of this
    # fixture's dialog, then the "Get Alert Text" command will return
    # successfully. In that case, the text must be different than that
    # of this fixture's dialog.
    try:
        assert_error(result, "no such alert")
    except:
        assert (result.status == 200 and
                result.body["value"] != expected_text), (
               "Dialog with text '%s' was not handled." % expected_text)

def assert_same_element(session, a, b):
    """Verify that two element references describe the same element."""
    assert isinstance(a, dict), "Actual value is not a dictionary"
    assert isinstance(b, dict), "Expected value is not a dictionary"
    assert element_key in a, "Actual value does not describe an element"
    assert element_key in b, "Expected value does not describe an element"

    if a[element_key] == b[element_key]:
        return

    message = ("Expected element references to describe the same element, " +
        "but they did not.")

    # Attempt to provide more information, accounting for possible errors such
    # as stale element references or not visible elements.
    try:
        a_markup = session.execute_script("return arguments[0].outerHTML;", args=[a])
        b_markup = session.execute_script("return arguments[0].outerHTML;", args=[b])
        message += " Actual: `%s`. Expected: `%s`." % (a_markup, b_markup)
    except WebDriverException:
        pass

    raise AssertionError(message)
