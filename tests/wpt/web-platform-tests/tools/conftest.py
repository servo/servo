import platform
import os
import sys

from hypothesis import settings, HealthCheck

impl = platform.python_implementation()

settings.register_profile("ci", settings(max_examples=1000,
                                         suppress_health_check=[HealthCheck.too_slow]))
settings.register_profile("pypy", settings(suppress_health_check=[HealthCheck.too_slow]))

settings.load_profile(os.getenv("HYPOTHESIS_PROFILE",
                                "default" if impl != "PyPy" else "pypy"))

# these can't even be imported on Py3, so totally ignore it even from collection
ignore_dirs = ["serve", "wptserve"]

collect_ignore = []
if sys.version_info[0] >= 3:
    for d in ignore_dirs:
        path = os.path.join(os.path.dirname(__file__), d)
        collect_ignore.extend([os.path.join(root, f)
                               for root, _, files in os.walk(path)
                               for f in files])
