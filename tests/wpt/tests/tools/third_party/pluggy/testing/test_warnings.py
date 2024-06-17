from pathlib import Path

import pytest

from pluggy import HookimplMarker
from pluggy import HookspecMarker
from pluggy import PluggyTeardownRaisedWarning
from pluggy import PluginManager


hookspec = HookspecMarker("example")
hookimpl = HookimplMarker("example")


def test_teardown_raised_warning(pm: PluginManager) -> None:
    class Api:
        @hookspec
        def my_hook(self):
            raise NotImplementedError()

    pm.add_hookspecs(Api)

    class Plugin1:
        @hookimpl
        def my_hook(self):
            pass

    class Plugin2:
        @hookimpl(hookwrapper=True)
        def my_hook(self):
            yield
            1 / 0

    class Plugin3:
        @hookimpl(hookwrapper=True)
        def my_hook(self):
            yield

    pm.register(Plugin1(), "plugin1")
    pm.register(Plugin2(), "plugin2")
    pm.register(Plugin3(), "plugin3")
    with pytest.warns(
        PluggyTeardownRaisedWarning,
        match=r"\bplugin2\b.*\bmy_hook\b.*\n.*ZeroDivisionError",
    ) as wc:
        with pytest.raises(ZeroDivisionError):
            pm.hook.my_hook()
    assert len(wc.list) == 1
    assert Path(wc.list[0].filename).name == "test_warnings.py"
