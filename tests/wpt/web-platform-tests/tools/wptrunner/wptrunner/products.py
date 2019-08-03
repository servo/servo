import importlib
import imp

from .browsers import product_list


def products_enabled(config):
    names = config.get("products", {}).keys()
    if not names:
        return product_list
    else:
        return names


def product_module(config, product):
    if product not in products_enabled(config):
        raise ValueError("Unknown product %s" % product)

    path = config.get("products", {}).get(product, None)
    if path:
        module = imp.load_source('wptrunner.browsers.' + product, path)
    else:
        module = importlib.import_module("wptrunner.browsers." + product)

    if not hasattr(module, "__wptrunner__"):
        raise ValueError("Product module does not define __wptrunner__ variable")

    return module


class Product(object):
    def __init__(self, config, product):
        module = product_module(config, product)
        data = module.__wptrunner__
        self.name = product
        self.check_args = getattr(module, data["check_args"])
        self.browser_cls = getattr(module, data["browser"])
        self.get_browser_kwargs = getattr(module, data["browser_kwargs"])
        self.get_executor_kwargs = getattr(module, data["executor_kwargs"])
        self.env_options = getattr(module, data["env_options"])()
        self.get_env_extras = getattr(module, data["env_extras"])
        self.run_info_extras = (getattr(module, data["run_info_extras"])
                           if "run_info_extras" in data else lambda **kwargs:{})
        self.get_timeout_multiplier = getattr(module, data["timeout_multiplier"])

        self.executor_classes = {}
        for test_type, cls_name in data["executor"].iteritems():
            cls = getattr(module, cls_name)
            self.executor_classes[test_type] = cls


def load_product(config, product, load_cls=False):
    rv = Product(config, product)
    if not load_cls:
        return (rv.check_args,
                rv.browser_cls,
                rv.get_browser_kwargs,
                rv.executor_classes,
                rv.get_executor_kwargs,
                rv.env_options,
                rv.get_env_extras,
                rv.run_info_extras)
    return rv


def load_product_update(config, product):
    """Return tuple of (property_order, boolean_properties) indicating the
    run_info properties to use when constructing the expectation data for
    this product. None for either key indicates that the default keys
    appropriate for distinguishing based on platform will be used."""

    module = product_module(config, product)
    data = module.__wptrunner__

    update_properties = (getattr(module, data["update_properties"])()
                         if "update_properties" in data else {})

    return update_properties
