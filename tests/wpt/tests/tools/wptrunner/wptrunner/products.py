from __future__ import annotations

import importlib
import warnings
from dataclasses import dataclass
from typing import (
    TYPE_CHECKING,
    Any,
    Protocol,
    TypedDict,
    overload,
)

from .browsers import product_list
from .deprecated import deprecated

if TYPE_CHECKING:
    import sys
    from types import ModuleType

    from mozlog.structuredlog import StructuredLogger
    from wptserve.config import Config

    from .browsers import base as browsers_base
    from .environment import TestEnvironment
    from .executors.base import TestExecutor
    from .testloader import Subsuite

    if sys.version_info >= (3, 9):
        from collections.abc import Mapping, Sequence
    else:
        from typing import Mapping, Sequence

    if sys.version_info >= (3, 10):
        from typing import TypeAlias
    else:
        from typing_extensions import TypeAlias


JSON: TypeAlias = "Mapping[str, 'JSON'] | Sequence['JSON'] | str | int | float | bool | None"


class CheckArgs(Protocol):
    def __call__(self, **kwargs: Any) -> None:
        ...


class EnvExtras(Protocol):
    def __call__(self, **kwargs: Any) -> Sequence[object]:
        ...


class BrowserKwargs(Protocol):
    def __call__(
        self,
        logger: StructuredLogger,
        test_type: str,
        run_info_data: Mapping[str, JSON],
        *,
        config: Config,
        subsuite: Subsuite,
        **kwargs: Any,
    ) -> Mapping[str, object]:
        ...


class ExecutorKwargs(Protocol):
    def __call__(
        self,
        logger: StructuredLogger,
        test_type: str,
        test_environment: TestEnvironment,
        run_info_data: Mapping[str, JSON],
        *,
        subsuite: Subsuite,
        **kwargs: Any,
    ) -> Mapping[str, object]:
        ...


class RunInfoExtras(Protocol):
    def __call__(
        self, logger: StructuredLogger, **kwargs: Any
    ) -> Mapping[str, JSON]:
        ...


class TimeoutMultiplier(Protocol):
    def __call__(
        self, test_type: str, run_info_data: Mapping[str, JSON], **kwargs: Any
    ) -> float:
        ...


class _WptrunnerModuleDictOptional(TypedDict, total=False):
    run_info_extras: str
    update_properties: str


class WptrunnerModuleDict(_WptrunnerModuleDictOptional):
    product: str
    browser: str | Mapping[str | None, str]
    check_args: str
    browser_kwargs: str
    executor_kwargs: str
    env_options: str
    env_extras: str
    timeout_multiplier: str
    executor: Mapping[str, str]


def _product_module(product: str) -> ModuleType:
    if product not in product_list:
        raise ValueError(f"Unknown product {product!r}")

    module = importlib.import_module("wptrunner.browsers." + product)
    if not hasattr(module, "__wptrunner__"):
        raise ValueError("Product module does not define __wptrunner__ variable")

    return module


def default_run_info_extras(logger: StructuredLogger, **kwargs: Any) -> Mapping[str, JSON]:
    return {}


_legacy_product_msg = "Use Product.from_product_name(name) instead of Product(config, name)"


@dataclass
class Product:
    name: str
    browser_classes: Mapping[str | None, type[browsers_base.Browser]]
    check_args: CheckArgs
    get_browser_kwargs: BrowserKwargs
    get_executor_kwargs: ExecutorKwargs
    env_options: Mapping[str, Any]
    get_env_extras: EnvExtras
    get_timeout_multiplier: TimeoutMultiplier
    executor_classes: Mapping[str, type[TestExecutor]]
    run_info_extras: RunInfoExtras
    update_properties: tuple[Sequence[str], Mapping[str, Sequence[str]]]

    @overload
    @deprecated(_legacy_product_msg, category=None)
    def __init__(
        self,
        config: object,
        legacy_name: str,
        /,
        *,
        _do_not_use_allow_legacy_name_call: bool = False,
    ) -> None:
        ...

    @overload
    def __init__(
        self,
        name: str,
        *,
        browser_classes: Mapping[str | None, type[browsers_base.Browser]],
        check_args: CheckArgs,
        get_browser_kwargs: BrowserKwargs,
        get_executor_kwargs: ExecutorKwargs,
        env_options: Mapping[str, Any],
        get_env_extras: EnvExtras,
        get_timeout_multiplier: TimeoutMultiplier,
        executor_classes: Mapping[str, type[TestExecutor]],
        run_info_extras: None | RunInfoExtras = None,
        update_properties: None | tuple[Sequence[str], Mapping[str, Sequence[str]]] = None,
    ) -> None:
        ...

    def __init__(
        self,
        name: object,
        _legacy_name: None | str = None,
        *,
        browser_classes: None | Mapping[str | None, type[browsers_base.Browser]] = None,
        check_args: None | CheckArgs = None,
        get_browser_kwargs: None | BrowserKwargs = None,
        get_executor_kwargs: None | ExecutorKwargs = None,
        env_options: None | Mapping[str, Any] = None,
        get_env_extras: None | EnvExtras = None,
        get_timeout_multiplier: None | TimeoutMultiplier = None,
        executor_classes: None | Mapping[str, type[TestExecutor]] = None,
        run_info_extras: None | RunInfoExtras = None,
        update_properties: None | tuple[Sequence[str], Mapping[str, Sequence[str]]] = None,
        _do_not_use_allow_legacy_name_call: bool = False,
    ) -> None:
        if _legacy_name is None:
            assert isinstance(name, str)
        else:
            if not _do_not_use_allow_legacy_name_call:
                warnings.warn(_legacy_product_msg, category=DeprecationWarning, stacklevel=2)

            module = _product_module(_legacy_name)
            data: WptrunnerModuleDict = module.__wptrunner__

            name = data["product"]
            if name != _legacy_name:
                msg = f"Product {_legacy_name!r} calls itself {name!r}, which differs"
                raise ValueError(msg)
            browser_classes = (
                {None: getattr(module, data["browser"])}
                if isinstance(data["browser"], str)
                else {
                    key: getattr(module, value)
                    for key, value in data["browser"].items()
                }
            )
            check_args = getattr(module, data["check_args"])
            get_browser_kwargs = getattr(module, data["browser_kwargs"])
            get_executor_kwargs = getattr(module, data["executor_kwargs"])
            env_options = getattr(module, data["env_options"])()
            get_env_extras = getattr(module, data["env_extras"])
            get_timeout_multiplier = getattr(module, data["timeout_multiplier"])
            executor_classes = {
                test_type: getattr(module, cls_name)
                for test_type, cls_name in data["executor"].items()
            }
            run_info_extras = (
                getattr(module, data["run_info_extras"])
                if "run_info_extras" in data
                else None
            )
            update_properties = (
                getattr(module, data["update_properties"])()
                if "update_properties" in data
                else None
            )

        assert browser_classes is not None
        assert check_args is not None
        assert get_browser_kwargs is not None
        assert get_executor_kwargs is not None
        assert env_options is not None
        assert get_env_extras is not None
        assert get_timeout_multiplier is not None
        assert executor_classes is not None

        self.name = name
        self._browser_cls = browser_classes
        self.check_args = check_args
        self.get_browser_kwargs = get_browser_kwargs
        self.get_executor_kwargs = get_executor_kwargs
        self.env_options = env_options
        self.get_env_extras = get_env_extras
        self.get_timeout_multiplier = get_timeout_multiplier
        self.executor_classes = executor_classes

        if run_info_extras is not None:
            self.run_info_extras = run_info_extras
        else:
            self.run_info_extras = default_run_info_extras

        if update_properties is not None:
            self.update_properties = update_properties
        else:
            self.update_properties = (["product"], {})

    @classmethod
    def from_product_name(cls, name: str) -> Product:
        return cls(None, name, _do_not_use_allow_legacy_name_call=True)

    def get_browser_cls(self, test_type: str) -> type[browsers_base.Browser]:
        if test_type in self._browser_cls:
            return self._browser_cls[test_type]
        return self._browser_cls[None]
