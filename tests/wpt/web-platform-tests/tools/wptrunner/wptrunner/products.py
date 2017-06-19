import os
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
    here = os.path.join(os.path.split(__file__)[0])
    product_dir = os.path.join(here, "browsers")

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


def load_product(config, product):
    module = product_module(config, product)
    data = module.__wptrunner__

    check_args = getattr(module, data["check_args"])
    browser_cls = getattr(module, data["browser"])
    browser_kwargs = getattr(module, data["browser_kwargs"])
    executor_kwargs = getattr(module, data["executor_kwargs"])
    env_options = getattr(module, data["env_options"])()
    env_extras = getattr(module, data["env_extras"])
    run_info_extras = (getattr(module, data["run_info_extras"])
                       if "run_info_extras" in data else lambda **kwargs:{})

    executor_classes = {}
    for test_type, cls_name in data["executor"].iteritems():
        cls = getattr(module, cls_name)
        executor_classes[test_type] = cls

    return (check_args,
            browser_cls, browser_kwargs,
            executor_classes, executor_kwargs,
            env_options, env_extras, run_info_extras)


def load_product_update(config, product):
    """Return tuple of (property_order, boolean_properties) indicating the
    run_info properties to use when constructing the expectation data for
    this product. None for either key indicates that the default keys
    appropriate for distinguishing based on platform will be used."""

    module = product_module(config, product)
    data = module.__wptrunner__

    update_properties = (getattr(module, data["update_properties"])()
                         if "update_properties" in data else (None, None))

    return update_properties
