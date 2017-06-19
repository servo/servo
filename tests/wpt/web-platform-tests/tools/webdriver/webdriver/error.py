import collections


class WebDriverException(Exception):
    http_status = None
    status_code = None


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
    status_code = "invalid cookie domain"


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
    http_status = 400
    status_code = "no such alert"


class NoSuchElementException(WebDriverException):
    http_status = 404
    status_code = "no such element"


class NoSuchFrameException(WebDriverException):
    http_status = 400
    status_code = "no such frame"


class NoSuchWindowException(WebDriverException):
    http_status = 400
    status_code = "no such window"


class ScriptTimeoutException(WebDriverException):
    http_status = 408
    status_code = "script timeout"


class SessionNotCreatedException(WebDriverException):
    http_status = 500
    status_code = "session not created"


class StaleElementReferenceException(WebDriverException):
    http_status = 400
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


def get(status_code):
    """Gets exception from `status_code`, falling back to
    ``WebDriverException`` if it is not found.
    """
    return _errors.get(status_code, WebDriverException)


_errors = collections.defaultdict()
for item in locals().values():
    if type(item) == type and issubclass(item, WebDriverException):
        _errors[item.status_code] = item
