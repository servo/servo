import pytest

from exceptiongroup import BaseExceptionGroup, ExceptionGroup, catch


def test_bad_arg():
    with pytest.raises(TypeError, match="the argument must be a mapping"):
        with catch(1):
            pass


def test_bad_handler():
    with pytest.raises(TypeError, match="handlers must be callable"):
        with catch({RuntimeError: None}):
            pass


@pytest.mark.parametrize(
    "exc_type",
    [
        pytest.param(BaseExceptionGroup, id="naked_basegroup"),
        pytest.param(ExceptionGroup, id="naked_group"),
        pytest.param((ValueError, BaseExceptionGroup), id="iterable_basegroup"),
        pytest.param((ValueError, ExceptionGroup), id="iterable_group"),
    ],
)
def test_catch_exceptiongroup(exc_type):
    with pytest.raises(TypeError, match="catching ExceptionGroup with catch"):
        with catch({exc_type: (lambda e: True)}):
            pass


def test_catch_ungrouped():
    value_type_errors = []
    zero_division_errors = []
    for exc in [ValueError("foo"), TypeError("bar"), ZeroDivisionError()]:
        with catch(
            {
                (ValueError, TypeError): value_type_errors.append,
                ZeroDivisionError: zero_division_errors.append,
            }
        ):
            raise exc

    assert len(value_type_errors) == 2

    assert isinstance(value_type_errors[0], ExceptionGroup)
    assert len(value_type_errors[0].exceptions) == 1
    assert isinstance(value_type_errors[0].exceptions[0], ValueError)

    assert isinstance(value_type_errors[1], ExceptionGroup)
    assert len(value_type_errors[1].exceptions) == 1
    assert isinstance(value_type_errors[1].exceptions[0], TypeError)

    assert len(zero_division_errors) == 1
    assert isinstance(zero_division_errors[0], ExceptionGroup)
    assert isinstance(zero_division_errors[0].exceptions[0], ZeroDivisionError)
    assert len(zero_division_errors[0].exceptions) == 1


def test_catch_group():
    value_runtime_errors = []
    zero_division_errors = []
    with catch(
        {
            (ValueError, RuntimeError): value_runtime_errors.append,
            ZeroDivisionError: zero_division_errors.append,
        }
    ):
        raise ExceptionGroup(
            "booboo",
            [
                ValueError("foo"),
                ValueError("bar"),
                RuntimeError("bar"),
                ZeroDivisionError(),
            ],
        )

    assert len(value_runtime_errors) == 1
    assert isinstance(value_runtime_errors[0], ExceptionGroup)
    exceptions = value_runtime_errors[0].exceptions
    assert isinstance(exceptions[0], ValueError)
    assert isinstance(exceptions[1], ValueError)
    assert isinstance(exceptions[2], RuntimeError)

    assert len(zero_division_errors) == 1
    assert isinstance(zero_division_errors[0], ExceptionGroup)
    exceptions = zero_division_errors[0].exceptions
    assert isinstance(exceptions[0], ZeroDivisionError)


def test_catch_nested_group():
    value_runtime_errors = []
    zero_division_errors = []
    with catch(
        {
            (ValueError, RuntimeError): value_runtime_errors.append,
            ZeroDivisionError: zero_division_errors.append,
        }
    ):
        nested_group = ExceptionGroup(
            "nested", [RuntimeError("bar"), ZeroDivisionError()]
        )
        raise ExceptionGroup("booboo", [ValueError("foo"), nested_group])

    assert len(value_runtime_errors) == 1
    exceptions = value_runtime_errors[0].exceptions
    assert isinstance(exceptions[0], ValueError)
    assert isinstance(exceptions[1], ExceptionGroup)
    assert isinstance(exceptions[1].exceptions[0], RuntimeError)

    assert len(zero_division_errors) == 1
    assert isinstance(zero_division_errors[0], ExceptionGroup)
    assert isinstance(zero_division_errors[0].exceptions[0], ExceptionGroup)
    assert isinstance(
        zero_division_errors[0].exceptions[0].exceptions[0], ZeroDivisionError
    )


def test_catch_no_match():
    try:
        with catch({(ValueError, RuntimeError): (lambda e: None)}):
            group = ExceptionGroup("booboo", [ZeroDivisionError()])
            raise group
    except ExceptionGroup as exc:
        assert exc is not group
    else:
        pytest.fail("Did not raise an ExceptionGroup")


def test_catch_single_no_match():
    try:
        with catch({(ValueError, RuntimeError): (lambda e: None)}):
            raise ZeroDivisionError
    except ZeroDivisionError:
        pass
    else:
        pytest.fail("Did not raise an ZeroDivisionError")


def test_catch_full_match():
    with catch({(ValueError, RuntimeError): (lambda e: None)}):
        raise ExceptionGroup("booboo", [ValueError()])


def test_catch_handler_raises():
    def handler(exc):
        raise RuntimeError("new")

    with pytest.raises(RuntimeError, match="new") as exc:
        with catch({(ValueError, ValueError): handler}):
            excgrp = ExceptionGroup("booboo", [ValueError("bar")])
            raise excgrp

    context = exc.value.__context__
    assert isinstance(context, ExceptionGroup)
    assert str(context) == "booboo (1 sub-exception)"
    assert len(context.exceptions) == 1
    assert isinstance(context.exceptions[0], ValueError)
    assert exc.value.__cause__ is None


def test_bare_raise_in_handler():
    """Test that a bare "raise"  "middle" ecxeption group gets discarded."""

    def handler(exc):
        raise

    with pytest.raises(ExceptionGroup) as excgrp:
        with catch({(ValueError,): handler, (RuntimeError,): lambda eg: None}):
            try:
                first_exc = RuntimeError("first")
                raise first_exc
            except RuntimeError as exc:
                middle_exc = ExceptionGroup(
                    "bad", [ValueError(), ValueError(), TypeError()]
                )
                raise middle_exc from exc

    assert len(excgrp.value.exceptions) == 2
    assert all(isinstance(exc, ValueError) for exc in excgrp.value.exceptions)
    assert excgrp.value is not middle_exc
    assert excgrp.value.__cause__ is first_exc
    assert excgrp.value.__context__ is first_exc


def test_catch_subclass():
    lookup_errors = []
    with catch({LookupError: lookup_errors.append}):
        raise KeyError("foo")

    assert len(lookup_errors) == 1
    assert isinstance(lookup_errors[0], ExceptionGroup)
    exceptions = lookup_errors[0].exceptions
    assert isinstance(exceptions[0], KeyError)


def test_async_handler(request):
    async def handler(eg):
        pass

    def delegate(eg):
        coro = handler(eg)
        request.addfinalizer(coro.close)
        return coro

    with pytest.raises(TypeError, match="Exception handler must be a sync function."):
        with catch({TypeError: delegate}):
            raise ExceptionGroup("message", [TypeError("uh-oh")])


def test_bare_reraise_from_naked_exception():
    def handler(eg):
        raise

    with pytest.raises(ExceptionGroup) as excgrp, catch({Exception: handler}):
        raise KeyError("foo")

    assert len(excgrp.value.exceptions) == 1
    assert isinstance(excgrp.value.exceptions[0], KeyError)
    assert str(excgrp.value.exceptions[0]) == "'foo'"
