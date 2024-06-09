# mypy: allow-untyped-defs

import collections
import json

from typing import ClassVar, DefaultDict, Type


class WebDriverException(Exception):
    # The status_code class variable is used to map the JSON Error Code (see
    # https://w3c.github.io/webdriver/#errors) to a WebDriverException subclass.
    # However, http_status need not match, and both are set as instance
    # variables, shadowing the class variables. TODO: Match on both http_status
    # and status_code and let these be class variables only.
    http_status: ClassVar[int]
    status_code: ClassVar[str]

    def __init__(self, http_status=None, status_code=None, message=None, stacktrace=None):
        super().__init__()

        if http_status is not None:
            self.http_status = http_status
        if status_code is not None:
            self.status_code = status_code
        self.message = message
        self.stacktrace = stacktrace

    def __repr__(self):
        return f"<{self.__class__.__name__} http_status={self.http_status}>"

    def __str__(self):
        message = f"{self.status_code} ({self.http_status})"

        if self.message is not None:
            message += ": %s" % self.message
        message += "\n"

        if self.stacktrace:
            message += ("\nRemote-end stacktrace:\n\n%s" % self.stacktrace)

        return message


class DetachedShadowRootException(WebDriverException):
    http_status = 404
    status_code = "detached shadow root"


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


class NoSuchCookieException(WebDriverException):
    http_status = 404
    status_code = "no such cookie"


class NoSuchElementException(WebDriverException):
    http_status = 404
    status_code = "no such element"


class NoSuchFrameException(WebDriverException):
    http_status = 404
    status_code = "no such frame"


class NoSuchShadowRootException(WebDriverException):
    http_status = 404
    status_code = "no such shadow root"


class NoSuchWindowException(WebDriverException):
    http_status = 404
    status_code = "no such window"


class ScriptTimeoutException(WebDriverException):
    http_status = 500
    status_code = "script timeout"


class SessionNotCreatedException(WebDriverException):
    http_status = 500
    status_code = "session not created"


class StaleElementReferenceException(WebDriverException):
    http_status = 404
    status_code = "stale element reference"


class TimeoutException(WebDriverException):
    http_status = 500
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
            response.status,
            None,
            "Response is not an error:\n"
            "%s" % json.dumps(response.body))

    if "value" in response.body:
        value = response.body["value"]
    else:
        raise UnknownErrorException(
            response.status,
            None,
            "Expected 'value' key in response body:\n"
            "%s" % json.dumps(response.body))

    # all fields must exist, but stacktrace can be an empty string
    code = value["error"]
    message = value["message"]
    stack = value["stacktrace"] or None

    cls = get(code)
    return cls(response.status, code, message, stacktrace=stack)


def get(error_code):
    """
    Gets exception from `error_code`, falling back to
    ``WebDriverException`` if it is not found.
    """
    return _errors.get(error_code, WebDriverException)


_errors: DefaultDict[str, Type[WebDriverException]] = collections.defaultdict()
for item in list(locals().values()):
    if isinstance(item, type) and item != WebDriverException and issubclass(item, WebDriverException):
        _errors[item.status_code] = item
