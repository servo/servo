import pytest
import webdriver.bidi.error as error

from webdriver.bidi.modules.script import ContextTarget, RealmTarget, SerializationOptions

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


@pytest.mark.parametrize("serialization_options", [False, "_UNKNOWN_", 42, []])
async def test_params_serialization_options_invalid_type(bidi_session, top_context, serialization_options):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            serialization_options=serialization_options,
            target=ContextTarget(top_context["context"]),
            await_promise=True)


@pytest.mark.parametrize("max_dom_depth", [False, "_UNKNOWN_", {}, []])
async def test_params_max_dom_depth_invalid_type(bidi_session, top_context, max_dom_depth):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            serialization_options=SerializationOptions(max_dom_depth=max_dom_depth),
            target=ContextTarget(top_context["context"]),
            await_promise=True)


async def test_params_max_dom_depth_invalid_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            serialization_options=SerializationOptions(max_dom_depth=-1),
            target=ContextTarget(top_context["context"]),
            await_promise=True)


@pytest.mark.parametrize("max_object_depth", [False, "_UNKNOWN_", {}, []])
async def test_params_max_object_depth_invalid_type(bidi_session, top_context, max_object_depth):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            serialization_options=SerializationOptions(max_object_depth=max_object_depth),
            target=ContextTarget(top_context["context"]),
            await_promise=True)


async def test_params_max_object_depth_invalid_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            serialization_options=SerializationOptions(max_object_depth=-1),
            target=ContextTarget(top_context["context"]),
            await_promise=True)


@pytest.mark.parametrize("include_shadow_tree", [False, 42, {}, []])
async def test_params_include_shadow_tree_invalid_type(bidi_session, top_context, include_shadow_tree):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            serialization_options=SerializationOptions(include_shadow_tree=include_shadow_tree),
            target=ContextTarget(top_context["context"]),
            await_promise=True)


async def test_params_include_shadow_tree_invalid_value(
        bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            serialization_options=SerializationOptions(include_shadow_tree="foo"),
            target=ContextTarget(top_context["context"]),
            await_promise=True)


@pytest.mark.parametrize("user_activation", ["foo", 42, {}, []])
async def test_params_user_activation_invalid_type(bidi_session, top_context, user_activation):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.evaluate(
            expression="1 + 2",
            user_activation=user_activation,
            target=ContextTarget(top_context["context"]),
            await_promise=True)
