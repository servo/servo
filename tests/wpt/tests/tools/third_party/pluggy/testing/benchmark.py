"""
Benchmarking and performance tests.
"""
import pytest
from pluggy import HookspecMarker, HookimplMarker, PluginManager
from pluggy._hooks import HookImpl
from pluggy._callers import _multicall


hookspec = HookspecMarker("example")
hookimpl = HookimplMarker("example")


@hookimpl
def hook(arg1, arg2, arg3):
    return arg1, arg2, arg3


@hookimpl(hookwrapper=True)
def wrapper(arg1, arg2, arg3):
    yield


@pytest.fixture(params=[10, 100], ids="hooks={}".format)
def hooks(request):
    return [hook for i in range(request.param)]


@pytest.fixture(params=[10, 100], ids="wrappers={}".format)
def wrappers(request):
    return [wrapper for i in range(request.param)]


def test_hook_and_wrappers_speed(benchmark, hooks, wrappers):
    def setup():
        hook_name = "foo"
        hook_impls = []
        for method in hooks + wrappers:
            f = HookImpl(None, "<temp>", method, method.example_impl)
            hook_impls.append(f)
        caller_kwargs = {"arg1": 1, "arg2": 2, "arg3": 3}
        firstresult = False
        return (hook_name, hook_impls, caller_kwargs, firstresult), {}

    benchmark.pedantic(_multicall, setup=setup)


@pytest.mark.parametrize(
    ("plugins, wrappers, nesting"),
    [
        (1, 1, 0),
        (1, 1, 1),
        (1, 1, 5),
        (1, 5, 1),
        (1, 5, 5),
        (5, 1, 1),
        (5, 1, 5),
        (5, 5, 1),
        (5, 5, 5),
        (20, 20, 0),
        (100, 100, 0),
    ],
)
def test_call_hook(benchmark, plugins, wrappers, nesting):
    pm = PluginManager("example")

    class HookSpec:
        @hookspec
        def fun(self, hooks, nesting: int):
            yield

    class Plugin:
        def __init__(self, num):
            self.num = num

        def __repr__(self):
            return f"<Plugin {self.num}>"

        @hookimpl
        def fun(self, hooks, nesting: int):
            if nesting:
                hooks.fun(hooks=hooks, nesting=nesting - 1)

    class PluginWrap:
        def __init__(self, num):
            self.num = num

        def __repr__(self):
            return f"<PluginWrap {self.num}>"

        @hookimpl(hookwrapper=True)
        def fun(self):
            yield

    pm.add_hookspecs(HookSpec)

    for i in range(plugins):
        pm.register(Plugin(i), name=f"plug_{i}")
    for i in range(wrappers):
        pm.register(PluginWrap(i), name=f"wrap_plug_{i}")

    benchmark(pm.hook.fun, hooks=pm.hook, nesting=nesting)
