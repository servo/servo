import pytest

from pluggy import HookspecMarker
from pluggy import PluginManager


@pytest.fixture(
    params=[lambda spec: spec, lambda spec: spec()],
    ids=["spec-is-class", "spec-is-instance"],
)
def he_pm(request, pm: PluginManager) -> PluginManager:
    hookspec = HookspecMarker("example")

    class Hooks:
        @hookspec
        def he_method1(self, arg: int) -> int:
            return arg + 1

    pm.add_hookspecs(request.param(Hooks))
    return pm


@pytest.fixture
def pm() -> PluginManager:
    return PluginManager("example")
