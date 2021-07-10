import platform
import os

from hypothesis import settings, HealthCheck

impl = platform.python_implementation()

settings.register_profile("ci", settings(max_examples=1000,
                                         deadline=None,
                                         suppress_health_check=[HealthCheck.too_slow]))
settings.register_profile("pypy", settings(deadline=None,
                                           suppress_health_check=[HealthCheck.too_slow]))

settings.load_profile(os.getenv("HYPOTHESIS_PROFILE",
                                "default" if impl != "PyPy" else "pypy"))
