"""Definition of WebDriverException classes."""

def create_webdriver_exception_strict(status_code, message):
    """Create the appropriate WebDriverException given the status_code."""
    if status_code in _exceptions_strict:
        return _exceptions_strict[status_code](message)
    return UnknownStatusCodeException("[%s] %s" % (status_code, message))

def create_webdriver_exception_compatibility(status_code, message):
    """Create the appropriate WebDriverException given the status_code."""
    if status_code in _exceptions_compatibility:
        return _exceptions_compatibility[status_code](message)
    return UnknownStatusCodeException("[%s] %s" % (status_code, message))

class WebDriverException(Exception):
    """Base class for all WebDriverExceptions."""

class UnableToSetCookieException(WebDriverException):
    """A request to set a cookie's value could not be satisfied."""

class InvalidElementStateException(WebDriverException):
    """An element command could not be completed because the element is
    in an invalid state (e.g. attempting to click an element that is no
    longer attached to the DOM).
    """

class NoSuchElementException(WebDriverException):
    """An element could not be located on the page using the given
    search parameters.
    """

class TimeoutException(WebDriverException):
    """An operation did not complete before its timeout expired."""

class ElementNotSelectableException(InvalidElementStateException):
    """An attempt was made to select an element that cannot be selected."""

class ElementNotVisibleException(InvalidElementStateException):
    """An element command could not be completed because the element is
    not visible on the page.
    """

class ImeEngineActivationFailedException(WebDriverException):
    """An IME engine could not be started."""

class ImeNotAvailableException(ImeEngineActivationFailedException):
    """IME was not available."""

class InvalidCookieDomainException(UnableToSetCookieException):
    """An illegal attempt was made to set a cookie under a different
    domain than the current page.
    """

class InvalidElementCoordinatesException(WebDriverException):
    """The coordinates provided to an interactions operation are invalid."""

class InvalidSelectorException(NoSuchElementException):
    """Argument was an invalid selector (e.g. XPath/CSS)."""

class JavascriptErrorException(WebDriverException):
    """An error occurred while executing user supplied JavaScript."""

class MoveTargetOutOfBoundsException(InvalidElementStateException):
    """The target for mouse interaction is not in the browser's viewport
    and cannot be brought into that viewport.
    """

class NoSuchAlertException(WebDriverException):
    """An attempt was made to operate on a modal dialog when one was not open."""

class NoSuchFrameException(WebDriverException):
    """A request to switch to a frame could not be satisfied because
    the frame could not be found."""

class NoSuchWindowException(WebDriverException):
    """A request to switch to a different window could not be satisfied
    because the window could not be found.
    """

class ScriptTimeoutException(TimeoutException):
    """A script did not complete before its timeout expired."""

class SessionNotCreatedException(WebDriverException):
    """A new session could not be created."""

class StaleElementReferenceException(InvalidElementStateException):
    """An element command failed because the referenced element is no
    longer attached to the DOM.
    """

class UnexpectedAlertOpenException(WebDriverException):
    """A modal dialog was open, blocking this operation."""

class UnknownCommandException(WebDriverException):
    """A command could not be executed because the remote end is not
    aware of it.
    """

class UnknownErrorException(WebDriverException):
    """An unknown error occurred in the remote end while processing
    the command.
    """

class UnsupportedOperationException(WebDriverException):
    """Indicates that a command that should have executed properly
    cannot be supported for some reason.
    """

class UnknownStatusCodeException(WebDriverException):
    """Exception for all other status codes."""

_exceptions_strict = {
    "element not selectable": ElementNotSelectableException,
    "element not visible": ElementNotVisibleException,
    "ime engine activation failed": ImeEngineActivationFailedException,
    "ime not available": ImeNotAvailableException,
    "invalid cookie domain": InvalidCookieDomainException,
    "invalid element coordinates": InvalidElementCoordinatesException,
    "invalid element state": InvalidElementStateException,
    "invalid selector": InvalidSelectorException,
    "javascript error": JavascriptErrorException,
    "move target out of bounds": MoveTargetOutOfBoundsException,
    "no such alert": NoSuchAlertException,
    "no such element": NoSuchElementException,
    "no such frame": NoSuchFrameException,
    "no such window": NoSuchWindowException,
    "script timeout": ScriptTimeoutException,
    "session not created": SessionNotCreatedException,
    "stale element reference": StaleElementReferenceException,
    "success": None,
    "timeout": TimeoutException,
    "unable to set cookie": UnableToSetCookieException,
    "unexpected alert open": UnexpectedAlertOpenException,
    "unknown command": UnknownCommandException,
    "unknown error": UnknownErrorException,
    "unsupported operation": UnsupportedOperationException,
}

_exceptions_compatibility = {
    15: ElementNotSelectableException,
    11: ElementNotVisibleException,
    31: ImeEngineActivationFailedException,
    30: ImeNotAvailableException,
    24: InvalidCookieDomainException,
    29: InvalidElementCoordinatesException,
    12: InvalidElementStateException,
    19: InvalidSelectorException,
    32: InvalidSelectorException,
    17: JavascriptErrorException,
    34: MoveTargetOutOfBoundsException,
    27: NoSuchAlertException,
    7: NoSuchElementException,
    8: NoSuchFrameException,
    23: NoSuchWindowException,
    28: ScriptTimeoutException,
    6: SessionNotCreatedException,
    33: SessionNotCreatedException,
    10: StaleElementReferenceException,
    0: None, # success
    21: TimeoutException,
    25: UnableToSetCookieException,
    26: UnexpectedAlertOpenException,
    9: UnknownCommandException,
    13: UnknownErrorException,
    # "unsupported operation": UnsupportedOperationException
}
