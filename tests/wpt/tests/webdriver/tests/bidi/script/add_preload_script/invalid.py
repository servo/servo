import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("function_declaration", [None, False, 42, {}, []])
async def test_params_function_declaration_invalid_type(
    bidi_session, function_declaration
):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration=function_declaration
        ),


@pytest.mark.parametrize("arguments", [False, "SOME_STRING", 42, {}])
async def test_params_arguments_invalid_type(bidi_session, arguments):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=arguments,
        )


@pytest.mark.parametrize("argument", [False, "SOME_STRING", 42, {}, []])
async def test_params_arguments_entry_invalid_type(bidi_session, argument):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=[argument],
        )


async def test_params_arguments_entry_invalid_value(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=[{"type": "foo"}],
        )


@pytest.mark.parametrize("value", [None, False, "_UNKNOWN_", 42, []])
async def test_params_arguments_channel_value_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=[{"type": "channel", "value": value}],
        )


@pytest.mark.parametrize("channel", [None, False, 42, [], {}])
async def test_params_arguments_channel_id_invalid_type(bidi_session, channel):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=[{"type": "channel", "value": {"channel": channel}}],
        )


@pytest.mark.parametrize("ownership", [False, 42, {}, []])
async def test_params_arguments_channel_ownership_invalid_type(bidi_session, ownership):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=[{"type": "channel", "value": {"ownership": ownership}}],
        )


async def test_params_arguments_channel_ownership_invalid_value(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=[{"type": "channel", "value": {
                "ownership": "_UNKNOWN_"}}],
        )


@pytest.mark.parametrize("serialization_options", [False, "_UNKNOWN_", 42, []])
async def test_params_arguments_channel_serialization_options_invalid_type(
    bidi_session, serialization_options
):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=[
                {
                    "type": "channel",
                    "value": {"serializationOptions": serialization_options},
                }
            ],
        )


@pytest.mark.parametrize("max_dom_depth", [False, "_UNKNOWN_", {}, []])
async def test_params_arguments_channel_max_dom_depth_invalid_type(
    bidi_session, max_dom_depth
):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=[
                {
                    "type": "channel",
                    "value": {"serializationOptions": {"maxDomDepth": max_dom_depth}},
                }
            ],
        )


async def test_params_arguments_channel_max_dom_depth_invalid_value(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=[
                {
                    "type": "channel",
                    "value": {"serializationOptions": {"maxDomDepth": -1}},
                }
            ],
        )


@pytest.mark.parametrize("max_object_depth", [False, "_UNKNOWN_", {}, []])
async def test_params_arguments_channel_max_object_depth_invalid_type(
    bidi_session, max_object_depth
):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=[
                {
                    "type": "channel",
                    "value": {
                        "serializationOptions": {"maxObjectDepth": max_object_depth}
                    },
                }
            ],
        )


async def test_params_arguments_channel_max_object_depth_invalid_value(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=[
                {
                    "type": "channel",
                    "value": {"serializationOptions": {"maxObjectDepth": -1}},
                }
            ],
        )


@pytest.mark.parametrize("include_shadow_tree", [False, 42, {}, []])
async def test_params_arguments_channel_include_shadow_tree_invalid_type(
    bidi_session, include_shadow_tree
):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=[
                {
                    "type": "channel",
                    "value": {
                        "serializationOptions": {
                            "includeShadowTree": include_shadow_tree
                        }
                    },
                }
            ],
        )


async def test_params_arguments_channel_include_shadow_tree_invalid_value(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            arguments=[
                {
                    "type": "channel",
                    "value": {
                        "serializationOptions": {"includeShadowTree": "_UNKNOWN_"}
                    },
                }
            ],
        )


@pytest.mark.parametrize("contexts", [False, 42, '_UNKNOWN_', {}])
async def test_params_contexts_invalid_type(bidi_session, contexts):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            contexts=contexts
        ),


async def test_params_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            contexts=[]
        ),


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_contexts_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            contexts=[value]
        ),


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_params_contexts_context_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            contexts=[value]
        ),


async def test_params_contexts_context_non_top_level(bidi_session, new_tab, test_page_same_origin_frame):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_page_same_origin_frame,
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])

    assert len(contexts) == 1
    assert len(contexts[0]["children"]) == 1
    child_info = contexts[0]["children"][0]

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}",
            contexts=[child_info['context']]
        ),


@pytest.mark.parametrize("sandbox", [False, 42, {}, []])
async def test_params_sandbox_invalid_type(bidi_session, sandbox):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}", sandbox=sandbox
        ),
