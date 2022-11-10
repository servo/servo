import pytest

from webdriver.bidi.modules.script import ContextTarget, ScriptEvaluateResultException
from .. import assert_handle


@pytest.mark.asyncio
@pytest.mark.parametrize("result_ownership, should_contain_handle",
                         [("root", True), ("none", False), (None, False)])
async def test_throw_exception(bidi_session, top_context, result_ownership, should_contain_handle):
    with pytest.raises(ScriptEvaluateResultException) as exception:
        await bidi_session.script.call_function(
            function_declaration='()=>{throw {a:1}}',
            await_promise=False,
            result_ownership=result_ownership,
            target=ContextTarget(top_context["context"]))

    assert_handle(exception.value.result["exceptionDetails"]["exception"], should_contain_handle)


@pytest.mark.asyncio
@pytest.mark.parametrize("result_ownership, should_contain_handle",
                         [("root", True), ("none", False), (None, False)])
async def test_invalid_script(bidi_session, top_context, result_ownership, should_contain_handle):
    with pytest.raises(ScriptEvaluateResultException) as exception:
        await bidi_session.script.call_function(
            function_declaration="))) !!@@## some invalid JS script (((",
            await_promise=False,
            result_ownership=result_ownership,
            target=ContextTarget(top_context["context"]))

    assert_handle(exception.value.result["exceptionDetails"]["exception"], should_contain_handle)


@pytest.mark.asyncio
@pytest.mark.parametrize("result_ownership, should_contain_handle",
                         [("root", True), ("none", False), (None, False)])
async def test_rejected_promise(bidi_session, top_context, result_ownership, should_contain_handle):
    with pytest.raises(ScriptEvaluateResultException) as exception:
        await bidi_session.script.call_function(
            function_declaration="()=>{return Promise.reject({a:1})}",
            await_promise=True,
            result_ownership=result_ownership,
            target=ContextTarget(top_context["context"]))

    assert_handle(exception.value.result["exceptionDetails"]["exception"], should_contain_handle)


@pytest.mark.asyncio
@pytest.mark.parametrize("await_promise", [True, False])
@pytest.mark.parametrize("result_ownership, should_contain_handle",
                         [("root", True), ("none", False), (None, False)])
async def test_return_value(bidi_session, top_context, await_promise, result_ownership, should_contain_handle):
    result = await bidi_session.script.call_function(
        function_declaration="async function(){return {a:1}}",
        await_promise=await_promise,
        result_ownership=result_ownership,
        target=ContextTarget(top_context["context"]))

    assert_handle(result, should_contain_handle)
