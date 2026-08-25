# mypy: allow-untyped-defs
import enum
from functools import cached_property
from functools import partial
from functools import wraps
import sys
from typing import TYPE_CHECKING
from typing import Union

from _pytest.compat import _PytestWrapper
from _pytest.compat import assert_never
from _pytest.compat import get_real_func
from _pytest.compat import is_generator
from _pytest.compat import safe_getattr
from _pytest.compat import safe_isclass
from _pytest.outcomes import OutcomeException
from _pytest.pytester import Pytester
import pytest


if TYPE_CHECKING:
    from typing_extensions import Literal


def test_is_generator() -> None:
    def zap():
        yield  # pragma: no cover

    def foo():
        pass  # pragma: no cover

    assert is_generator(zap)
    assert not is_generator(foo)


def test_real_func_loop_limit() -> None:
    class Evil:
        def __init__(self):
            self.left = 1000

        def __repr__(self):
            return f"<Evil left={self.left}>"

        def __getattr__(self, attr):
            if not self.left:
                raise RuntimeError("it's over")  # pragma: no cover
            self.left -= 1
            return self

    evil = Evil()

    with pytest.raises(
        ValueError,
        match=(
            "could not find real function of <Evil left=800>\n"
            "stopped at <Evil left=800>"
        ),
    ):
        get_real_func(evil)


def test_get_real_func() -> None:
    """Check that get_real_func correctly unwraps decorators until reaching the real function"""

    def decorator(f):
        @wraps(f)
        def inner():
            pass  # pragma: no cover

        return inner

    def func():
        pass  # pragma: no cover

    wrapped_func = decorator(decorator(func))
    assert get_real_func(wrapped_func) is func

    wrapped_func2 = decorator(decorator(wrapped_func))
    assert get_real_func(wrapped_func2) is func

    # special case for __pytest_wrapped__ attribute: used to obtain the function up until the point
    # a function was wrapped by pytest itself
    wrapped_func2.__pytest_wrapped__ = _PytestWrapper(wrapped_func)
    assert get_real_func(wrapped_func2) is wrapped_func


def test_get_real_func_partial() -> None:
    """Test get_real_func handles partial instances correctly"""

    def foo(x):
        return x

    assert get_real_func(foo) is foo
    assert get_real_func(partial(foo)) is foo


@pytest.mark.skipif(sys.version_info >= (3, 11), reason="coroutine removed")
def test_is_generator_asyncio(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        from _pytest.compat import is_generator
        import asyncio
        @asyncio.coroutine
        def baz():
            yield from [1,2,3]

        def test_is_generator_asyncio():
            assert not is_generator(baz)
    """
    )
    # avoid importing asyncio into pytest's own process,
    # which in turn imports logging (#8)
    result = pytester.runpytest_subprocess()
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_is_generator_async_syntax(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        from _pytest.compat import is_generator
        def test_is_generator_py35():
            async def foo():
                await foo()

            async def bar():
                pass

            assert not is_generator(foo)
            assert not is_generator(bar)
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_is_generator_async_gen_syntax(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        from _pytest.compat import is_generator
        def test_is_generator():
            async def foo():
                yield
                await foo()

            async def bar():
                yield

            assert not is_generator(foo)
            assert not is_generator(bar)
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*1 passed*"])


class ErrorsHelper:
    @property
    def raise_baseexception(self):
        raise BaseException("base exception should be raised")

    @property
    def raise_exception(self):
        raise Exception("exception should be caught")

    @property
    def raise_fail_outcome(self):
        pytest.fail("fail should be caught")


def test_helper_failures() -> None:
    helper = ErrorsHelper()
    with pytest.raises(Exception):  # noqa: B017
        _ = helper.raise_exception
    with pytest.raises(OutcomeException):
        _ = helper.raise_fail_outcome


def test_safe_getattr() -> None:
    helper = ErrorsHelper()
    assert safe_getattr(helper, "raise_exception", "default") == "default"
    assert safe_getattr(helper, "raise_fail_outcome", "default") == "default"
    with pytest.raises(BaseException):  # noqa: B017
        assert safe_getattr(helper, "raise_baseexception", "default")


def test_safe_isclass() -> None:
    assert safe_isclass(type) is True

    class CrappyClass(Exception):
        # Type ignored because it's bypassed intentionally.
        @property  # type: ignore
        def __class__(self):
            assert False, "Should be ignored"

    assert safe_isclass(CrappyClass()) is False


def test_cached_property() -> None:
    ncalls = 0

    class Class:
        @cached_property
        def prop(self) -> int:
            nonlocal ncalls
            ncalls += 1
            return ncalls

    c1 = Class()
    assert ncalls == 0
    assert c1.prop == 1
    assert c1.prop == 1
    c2 = Class()
    assert ncalls == 1
    assert c2.prop == 2
    assert c1.prop == 1


def test_assert_never_union() -> None:
    x: Union[int, str] = 10

    if isinstance(x, int):
        pass
    else:
        with pytest.raises(AssertionError):
            assert_never(x)  # type: ignore[arg-type]

    if isinstance(x, int):
        pass
    elif isinstance(x, str):
        pass
    else:
        assert_never(x)


def test_assert_never_enum() -> None:
    E = enum.Enum("E", "a b")
    x: E = E.a

    if x is E.a:
        pass
    else:
        with pytest.raises(AssertionError):
            assert_never(x)  # type: ignore[arg-type]

    if x is E.a:
        pass
    elif x is E.b:
        pass
    else:
        assert_never(x)


def test_assert_never_literal() -> None:
    x: Literal["a", "b"] = "a"

    if x == "a":
        pass
    else:
        with pytest.raises(AssertionError):
            assert_never(x)  # type: ignore[arg-type]

    if x == "a":
        pass
    elif x == "b":
        pass
    else:
        assert_never(x)
