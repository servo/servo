import pytest

from webdriver.bidi.error import UnknownCommandException


@pytest.mark.asyncio
@pytest.mark.parametrize("module_name, command_name", [
    ("invalidmodule", "somecommand"),
    ("session", "wrongcommand"),
], ids=[
    'invalid module',
    'invalid command name',
])
async def test_unknown_command(send_blocking_command, module_name, command_name):
    with pytest.raises(UnknownCommandException):
        await send_blocking_command(f"{module_name}.{command_name}", {})
