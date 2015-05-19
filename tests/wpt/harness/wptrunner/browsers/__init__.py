# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

"""Subpackage where each product is defined. Each product is created by adding a
a .py file containing a __wptrunner__ variable in the global scope. This must be
a dictionary with the fields

"product": Name of the product, assumed to be unique.
"browser": String indicating the Browser implementation used to launch that
           product.
"executor": Dictionary with keys as supported test types and values as the name
            of the Executor implemantation that will be used to run that test
            type.
"browser_kwargs": String naming function that takes product, binary,
                  prefs_root and the wptrunner.run_tests kwargs dict as arguments
                  and returns a dictionary of kwargs to use when creating the
                  Browser class.
"executor_kwargs": String naming a function that takes http server url and
                   timeout multiplier and returns kwargs to use when creating
                   the executor class.
"env_options": String naming a funtion of no arguments that returns the
               arguments passed to the TestEnvironment.

All classes and functions named in the above dict must be imported into the
module global scope.
"""

product_list = ["b2g",
                "chrome",
                "firefox",
                "servo",
                "servodriver"]
