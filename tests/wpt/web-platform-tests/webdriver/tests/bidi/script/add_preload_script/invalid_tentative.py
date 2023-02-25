import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("function_declaration", [None, False, 42, {}, []])
async def test_params_function_declaration_invalid_type(bidi_session, function_declaration):
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


@pytest.mark.parametrize("sandbox", [False, 42, {}, []])
async def test_params_sandbox_invalid_type(bidi_session, sandbox):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.script.add_preload_script(
            function_declaration="() => {}", sandbox=sandbox
        ),
