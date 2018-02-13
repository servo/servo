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

# serve can't even be imported on Py3, so totally ignore it even from collection
collect_ignore = []
if sys.version_info[0] >= 3:
    serve = os.path.join(os.path.dirname(__file__), "serve")
    collect_ignore.extend([os.path.join(root, f)
                           for root, _, files in os.walk(serve)
                           for f in files])
