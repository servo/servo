import collections

from typing import ClassVar, DefaultDict, Optional, Type


class BidiException(Exception):
    # The error_code class variable is used to map the JSON Error Code (see
    # https://w3c.github.io/webdriver/#errors) to a BidiException subclass.
    # TODO: Match on error and let it be a class variables only.
    error_code = None  # type: ClassVar[str]

    def __init__(self, error: str, message: str, stacktrace: Optional[str]):
        super(BidiException, self)

        self.error = error
        self.message = message
        self.stacktrace = stacktrace

    def __repr__(self):
        """Return the object representation in string format."""
        return f"{self.__class__.__name__}({self.error}, {self.message}, {self.stacktrace})"

    def __str__(self):
        """Return the string representation of the object."""
        message = f"{self.error} ({self.message})"

        if self.stacktrace:
            message += f"\n\nRemote-end stacktrace:\n\n{self.stacktrace}"

        return message


class InvalidArgumentException(BidiException):
    error_code = "invalid argument"


class UnknownCommandException(BidiException):
    error_code = "unknown command"


class UnknownErrorException(BidiException):
    error_code = "unknown error"


def from_error_details(error: str, message: str, stacktrace: Optional[str]) -> BidiException:
    """Create specific WebDriver BiDi exception class from error details.

    Defaults to ``UnknownErrorException`` if `error` is unknown.
    """
    cls = get(error)
    return cls(error, message, stacktrace)


def get(error_code: str) -> Type[BidiException]:
    """Get exception from `error_code`.

    It's falling back to ``UnknownErrorException`` if it is not found.
    """
    return _errors.get(error_code, UnknownErrorException)


_errors: DefaultDict[str, Type[BidiException]] = collections.defaultdict()
for item in list(locals().values()):
    if type(item) == type and issubclass(item, BidiException):
        _errors[item.error_code] = item
