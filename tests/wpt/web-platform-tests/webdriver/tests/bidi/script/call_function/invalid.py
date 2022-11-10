import pytest
import webdriver.bidi.error as error

from webdriver.bidi.modules.script import ContextTarget, RealmTarget

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("target", [None, False, "foo", 42, {}, []])
async def test_params_target_invalid_type(bidi_session, target):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.call_function(
            function_declaration="1 + 2",
            await_promise=False,
            target=target)


@pytest.mark.parametrize("context", [None, False, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.call_function(
            function_declaration="1 + 2",
            await_promise=False,
            target=ContextTarget(context))


@pytest.mark.parametrize("sandbox", [False, 42, {}, []])
async def test_params_sandbox_invalid_type(bidi_session, top_context, sandbox):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.call_function(
            function_declaration="1 + 2",
            await_promise=False,
            target=ContextTarget(top_context["context"],
                                 sandbox))


async def test_params_context_unknown(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.script.call_function(
            function_declaration="1 + 2",
            await_promise=False,
            target=ContextTarget("_UNKNOWN_"))


@pytest.mark.parametrize("realm", [None, False, 42, {}, []])
async def test_params_realm_invalid_type(bidi_session, realm):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.call_function(
            function_declaration="1 + 2",
            await_promise=False,
            target=RealmTarget(realm))


async def test_params_realm_unknown(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.script.call_function(
            function_declaration="1 + 2",
            await_promise=False,
            target=RealmTarget("_UNKNOWN_"))


@pytest.mark.parametrize("function_declaration", [None, False, 42, {}, []])
async def test_params_function_declaration_invalid_type(bidi_session, top_context,
                                                        function_declaration):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.call_function(
            function_declaration=function_declaration,
            await_promise=False,
            target=ContextTarget(top_context["context"]))


@pytest.mark.parametrize("this", [False, "SOME_STRING", 42, {}, []])
async def test_params_this_invalid_type(bidi_session, top_context,
                                        this):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.call_function(
            function_declaration="1 + 2",
            this=this,
            await_promise=False,
            target=ContextTarget(top_context["context"]))


@pytest.mark.parametrize("arguments", [False, "SOME_STRING", 42, {}])
async def test_params_arguments_invalid_type(bidi_session, top_context,
                                             arguments):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.call_function(
            function_declaration="1 + 2",
            arguments=arguments,
            await_promise=False,
            target=ContextTarget(top_context["context"]))


@pytest.mark.parametrize("argument", [False, "SOME_STRING", 42, {}, []])
async def test_params_single_argument_invalid_type(bidi_session, top_context,
                                                   argument):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.call_function(
            function_declaration="1 + 2",
            arguments=[argument],
            await_promise=False,
            target=ContextTarget(top_context["context"]))


@pytest.mark.parametrize("await_promise", [None, "False", 0, 42, {}, []])
async def test_params_await_promise_invalid_type(bidi_session, top_context,
                                                 await_promise):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.call_function(
            function_declaration="1 + 2",
            await_promise=await_promise,
            target=ContextTarget(top_context["context"]))


@pytest.mark.parametrize("result_ownership", [False, "_UNKNOWN_", 42, {}, []])
async def test_params_result_ownership_invalid_value(bidi_session, top_context,
                                                     result_ownership):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.call_function(
            function_declaration="1 + 2",
            await_promise=False,
            target=ContextTarget(top_context["context"]),
            result_ownership=result_ownership)
