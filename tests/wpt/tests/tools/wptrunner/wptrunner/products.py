# mypy: allow-untyped-defs
import importlib

from .browsers import product_list


def product_module(config, product):
    if product not in product_list:
        raise ValueError("Unknown product %s" % product)

    module = importlib.import_module("wptrunner.browsers." + product)
    if not hasattr(module, "__wptrunner__"):
        raise ValueError("Product module does not define __wptrunner__ variable")

    return module


class Product:
    def __init__(self, config, product):
        module = product_module(config, product)
        data = module.__wptrunner__
        self.name = product
        if isinstance(data["browser"], str):
            self._browser_cls = {None: getattr(module, data["browser"])}
        else:
            self._browser_cls = {key: getattr(module, value)
                                 for key, value in data["browser"].items()}
        self.check_args = getattr(module, data["check_args"])
        self.get_browser_kwargs = getattr(module, data["browser_kwargs"])
        self.get_executor_kwargs = getattr(module, data["executor_kwargs"])
        self.env_options = getattr(module, data["env_options"])()
        self.get_env_extras = getattr(module, data["env_extras"])
        self.run_info_extras = (getattr(module, data["run_info_extras"])
                                if "run_info_extras" in data else lambda product, **kwargs:{})
        self.get_timeout_multiplier = getattr(module, data["timeout_multiplier"])

        self.executor_classes = {}
        for test_type, cls_name in data["executor"].items():
            cls = getattr(module, cls_name)
            self.executor_classes[test_type] = cls

        self.update_properties = (getattr(module, data["update_properties"])()
                                  if "update_properties" in data else (["product"], {}))


    def get_browser_cls(self, test_type):
        if test_type in self._browser_cls:
            return self._browser_cls[test_type]
        return self._browser_cls[None]
