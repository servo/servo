import pytest


@pytest.fixture(
    params=[
        lambda spec: spec,
        lambda spec: spec()
    ],
    ids=[
        "spec-is-class",
        "spec-is-instance"
    ],
)
def he_pm(request, pm):
    from pluggy import HookspecMarker
    hookspec = HookspecMarker("example")

    class Hooks(object):
        @hookspec
        def he_method1(self, arg):
            return arg + 1

    pm.add_hookspecs(request.param(Hooks))
    return pm


@pytest.fixture
def pm():
    from pluggy import PluginManager
    return PluginManager("example")
