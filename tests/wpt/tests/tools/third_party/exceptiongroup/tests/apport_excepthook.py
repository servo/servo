# The apport_python_hook package is only installed as part of Ubuntu's system
# python, and not available in venvs. So before we can import it we have to
# make sure it's on sys.path.
import sys

sys.path.append("/usr/lib/python3/dist-packages")
import apport_python_hook  # unsorted import

apport_python_hook.install()

from exceptiongroup import ExceptionGroup  # noqa: E402 # unsorted import

raise ExceptionGroup("msg1", [KeyError("msg2"), ValueError("msg3")])
