import pytest
import webdriver.bidi.error as error

from webdriver.bidi.modules.script import ContextTarget, RealmTarget

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("target", [None, False, "foo", 42, {}, []])
async def test_params_target_invalid_type(bidi_session, target):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            target=target,
            await_promise=True)


@pytest.mark.parametrize("context", [None, False, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            target=ContextTarget(context),
            await_promise=True)


@pytest.mark.parametrize("sandbox", [False, 42, {}, []])
async def test_params_sandbox_invalid_type(bidi_session, top_context, sandbox):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            target=ContextTarget(top_context["context"], sandbox),
            await_promise=True)


async def test_params_context_unknown(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            target=ContextTarget("_UNKNOWN_"),
            await_promise=True)


@pytest.mark.parametrize("realm", [None, False, 42, {}, []])
async def test_params_realm_invalid_type(bidi_session, realm):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            target=RealmTarget(realm),
            await_promise=True)


async def test_params_realm_unknown(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            target=RealmTarget("_UNKNOWN_"),
            await_promise=True)


@pytest.mark.parametrize("expression", [None, False, 42, {}, []])
async def test_params_expression_invalid_type(bidi_session, top_context, expression):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression=expression,
            target=ContextTarget(top_context["context"]),
            await_promise=True)


@pytest.mark.parametrize("await_promise", [None, "False", 0, 42, {}, []])
async def test_params_await_promise_invalid_type(bidi_session, top_context, await_promise):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            await_promise=await_promise,
            target=ContextTarget(top_context["context"]))


@pytest.mark.parametrize("result_ownership", [False, "_UNKNOWN_", 42, {}, []])
async def test_params_result_ownership_invalid_value(bidi_session, top_context, result_ownership):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            result_ownership=result_ownership,
            target=ContextTarget(top_context["context"]),
            await_promise=True)
