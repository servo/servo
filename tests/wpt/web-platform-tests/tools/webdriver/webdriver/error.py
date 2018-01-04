import collections
import json


class WebDriverException(Exception):
    http_status = None
    status_code = None

    def __init__(self, message, stacktrace=None):
        super(WebDriverException, self)
        self.message = message
        self.stacktrace = stacktrace

    def __repr__(self):
        return "<%s http_status=%s>" % (self.__class__.__name__, self.http_status)

    def __str__(self):
        message = "%s (%s): %s\n" % (self.status_code, self.http_status, self.message)
        if self.stacktrace:
            message += ("\n"
            "Remote-end stacktrace:\n"
            "\n"
            "%s" % self.stacktrace)
        return message


class ElementClickInterceptedException(WebDriverException):
    http_status = 400
    status_code = "element click intercepted"


class ElementNotSelectableException(WebDriverException):
    http_status = 400
    status_code = "element not selectable"


class ElementNotVisibleException(WebDriverException):
    http_status = 400
    status_code = "element not visible"


class InsecureCertificateException(WebDriverException):
    http_status = 400
    status_code = "insecure certificate"


class InvalidArgumentException(WebDriverException):
    http_status = 400
    status_code = "invalid argument"


class InvalidCookieDomainException(WebDriverException):
    http_status = 400
    status_code = "invalid cookie domain"


class InvalidElementCoordinatesException(WebDriverException):
    http_status = 400
    status_code = "invalid element coordinates"


class InvalidElementStateException(WebDriverException):
    http_status = 400
    status_code = "invalid element state"


class InvalidSelectorException(WebDriverException):
    http_status = 400
    status_code = "invalid selector"


class InvalidSessionIdException(WebDriverException):
    http_status = 404
    status_code = "invalid session id"


class JavascriptErrorException(WebDriverException):
    http_status = 500
    status_code = "javascript error"


class MoveTargetOutOfBoundsException(WebDriverException):
    http_status = 500
    status_code = "move target out of bounds"


class NoSuchAlertException(WebDriverException):
    http_status = 404
    status_code = "no such alert"


class NoSuchElementException(WebDriverException):
    http_status = 404
    status_code = "no such element"


class NoSuchFrameException(WebDriverException):
    http_status = 404
    status_code = "no such frame"


class NoSuchWindowException(WebDriverException):
    http_status = 404
    status_code = "no such window"


class ScriptTimeoutException(WebDriverException):
    http_status = 408
    status_code = "script timeout"


class SessionNotCreatedException(WebDriverException):
    http_status = 500
    status_code = "session not created"


class StaleElementReferenceException(WebDriverException):
    http_status = 404
    status_code = "stale element reference"


class TimeoutException(WebDriverException):
    http_status = 408
    status_code = "timeout"


class UnableToSetCookieException(WebDriverException):
    http_status = 500
    status_code = "unable to set cookie"


class UnexpectedAlertOpenException(WebDriverException):
    http_status = 500
    status_code = "unexpected alert open"


class UnknownErrorException(WebDriverException):
    http_status = 500
    status_code = "unknown error"


class UnknownCommandException(WebDriverException):
    http_status = 404
    status_code = "unknown command"


class UnknownMethodException(WebDriverException):
    http_status = 405
    status_code = "unknown method"


class UnsupportedOperationException(WebDriverException):
    http_status = 500
    status_code = "unsupported operation"


def from_response(response):
    """
    Unmarshals an error from a ``Response``'s `body`, failing
    if not all three required `error`, `message`, and `stacktrace`
    fields are given.  Defaults to ``WebDriverException`` if `error`
    is unknown.
    """
    if response.status == 200:
        raise UnknownErrorException(
            "Response is not an error:\n"
            "%s" % json.dumps(response.body))

    if "value" in response.body:
        value = response.body["value"]
    else:
        raise UnknownErrorException(
            "Expected 'value' key in response body:\n"
            "%s" % json.dumps(response.body))

    # all fields must exist, but stacktrace can be an empty string
    code = value["error"]
    message = value["message"]
    stack = value["stacktrace"] or None

    cls = get(code)
    return cls(message, stacktrace=stack)


def get(error_code):
    """
    Gets exception from `error_code`, falling back to
    ``WebDriverException`` if it is not found.
    """
    return _errors.get(error_code, WebDriverException)


_errors = collections.defaultdict()
for item in locals().values():
    if type(item) == type and issubclass(item, WebDriverException):
        _errors[item.status_code] = item
