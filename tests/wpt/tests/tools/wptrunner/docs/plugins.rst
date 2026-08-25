Product Plugins
===============

External packages can register custom browser products with wptrunner using
entry points. Products registered this way are automatically
available to both the wptrunner test harness and the ``./wpt run`` CLI command.

Creating a Product Plugin
-------------------------

A product plugin consists of three components:

* A Python package containing your product implementation
* A ``get_product()`` function that returns a :py:class:`~products.Product` instance
* An entry point registration in ``setup.py`` or ``pyproject.toml``

Entry Point Registration
~~~~~~~~~~~~~~~~~~~~~~~~~

Register via the ``wptrunner.products`` entry point group. Example ``setup.py``::

    setup(
        name='wptrunner-mybrowser',
        install_requires=['wptrunner'],
        entry_points={
            'wptrunner.products': ['mybrowser = wptrunner_mybrowser:get_product']
        }
    )

Or ``pyproject.toml``::

    [project.entry-points."wptrunner.products"]
    mybrowser = "wptrunner_mybrowser:get_product"

Product Module Implementation
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Implement a Browser class and helper functions::

    from wptrunner.products import Product
    from wptrunner.browsers.base import Browser
    from wptrunner.executors.executorwebdriver import (
        WebDriverTestharnessExecutor,
        WebDriverRefTestExecutor,
    )

    class MyBrowser(Browser):
        def start(self, **kwargs):
            # Launch browser process
            # ...

        def stop(self, force=False):
            # ...

        def is_alive(self):
            # ...

        def executor_browser(self):
            from wptrunner.executors.executorwebdriver import WebDriverBrowser
            return WebDriverBrowser, {"webdriver_url": self.webdriver_url}

    def check_args(**kwargs):
        pass  # Validate required arguments

    def browser_kwargs(logger, test_type, run_info_data, config, subsuite, **kwargs):
        return {"binary": kwargs.get("binary")}

    def executor_kwargs(logger, test_type, test_environment, run_info_data,
                        subsuite, **kwargs):
        return {"capabilities": {"browserName": "mybrowser"}}

    def get_product():
        return Product(
            name="mybrowser",
            browser_classes={None: MyBrowser},
            check_args=check_args,
            get_browser_kwargs=browser_kwargs,
            get_executor_kwargs=executor_kwargs,
            env_options={"host": "web-platform.test", "bind_address": True},
            get_env_extras=lambda **kw: [],
            get_timeout_multiplier=lambda *a, **kw: 1.0,
            executor_classes={
                "testharness": WebDriverTestharnessExecutor,
                "reftest": WebDriverRefTestExecutor,
            },
        )

Advanced Features
-----------------

Multi-Browser Support
~~~~~~~~~~~~~~~~~~~~~

Specify different browser classes per test type::

    browser_classes={
        None: MyBrowser,              # Default
        "wdspec": MyWdSpecBrowser,    # WebDriver spec tests
    }

Run Info Extras
~~~~~~~~~~~~~~~

Add custom metadata to test run information::

    def run_info_extras(logger, **kwargs):
        return {"mybrowser_version": "1.0.0"}

    # Include in Product(..., run_info_extras=run_info_extras)

Custom Command-Line Arguments
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Products can register their own command-line arguments that appear when users
run ``./wpt run --help``. Use the ``add_arguments`` attribute to add a function
that receives an ``argparse.ArgumentParser`` and adds product-specific options::

    def add_arguments(parser):
        group = parser.add_argument_group("MyBrowser-specific")
        group.add_argument(
            "--mybrowser-profile",
            help="Path to browser profile directory"
        )
        group.add_argument(
            "--mybrowser-debug",
            action="store_true",
            help="Enable debug mode"
        )

    # Include in Product(..., add_arguments=add_arguments)

These arguments will be available in your ``check_args``, ``browser_kwargs``,
and other functions via ``**kwargs``. The ``add_arguments`` function is called
for all available products during argument parser setup, so arguments are
always visible regardless of which product is selected.

API Reference
-------------

See the :py:class:`~products.Product` dataclass documentation for complete
field descriptions. All fields are documented in the Product class docstring,
including required and optional attributes, function signatures, and usage
examples.

Troubleshooting
---------------

Product Not Found
~~~~~~~~~~~~~~~~~

If your product doesn't appear in ``./wpt run --help``:

* Verify entry point is registered correctly
* Reinstall: ``pip uninstall wptrunner-mybrowser && pip install -e .``

Import Errors
~~~~~~~~~~~~~

Ensure all imports are available::

    from wptrunner.products import Product
    from wptrunner.browsers.base import Browser

Product Name Mismatch
~~~~~~~~~~~~~~~~~~~~~

Entry point name must match Product name::

    'mybrowser = ...'  # Entry point
    Product(name="mybrowser", ...)  # Product

For usage with ``./wpt run``, see the running-tests documentation.
