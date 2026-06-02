from typing_extensions import assert_type

from exceptiongroup import BaseExceptionGroup, ExceptionGroup, catch, suppress

# issue 117
a = BaseExceptionGroup("", (KeyboardInterrupt(),))
assert_type(a, BaseExceptionGroup[KeyboardInterrupt])
b = BaseExceptionGroup("", (ValueError(),))
assert_type(b, BaseExceptionGroup[ValueError])
c = ExceptionGroup("", (ValueError(),))
assert_type(c, ExceptionGroup[ValueError])

# expected type error when passing a BaseException to ExceptionGroup
ExceptionGroup("", (KeyboardInterrupt(),))  # type: ignore[type-var]


# code snippets from the README


def value_key_err_handler(excgroup: BaseExceptionGroup) -> None:
    for exc in excgroup.exceptions:
        print("Caught exception:", type(exc))


def runtime_err_handler(exc: BaseExceptionGroup) -> None:
    print("Caught runtime error")


with catch(
    {(ValueError, KeyError): value_key_err_handler, RuntimeError: runtime_err_handler}
):
    ...


with suppress(RuntimeError):
    raise ExceptionGroup("", [RuntimeError("boo")])
