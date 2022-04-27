try:
    from importlib import reload
except ImportError:
    pass
import json
import os
import queue
import tempfile
import threading

import pytest

from . import serve
from wptserve import logger


class ServerProcSpy(serve.ServerProc):
    instances = None

    def start(self, *args, **kwargs):
        result = super().start(*args, **kwargs)

        if ServerProcSpy.instances is not None:
            ServerProcSpy.instances.put(self)

        return result

serve.ServerProc = ServerProcSpy  # type: ignore

@pytest.fixture()
def server_subprocesses():
    ServerProcSpy.instances = queue.Queue()
    yield ServerProcSpy.instances
    ServerProcSpy.instances = None

@pytest.fixture()
def tempfile_name():
    fd, name = tempfile.mkstemp()
    yield name
    os.close(fd)
    os.remove(name)


def test_subprocess_exit(server_subprocesses, tempfile_name):
    timeout = 30

    def target():
        # By default, the server initially creates a child process to validate
        # local system configuration. That process is unrelated to the behavior
        # under test, but at the time of this writing, the parent uses the same
        # constructor that is also used to create the long-running processes
        # which are relevant to this functionality. Disable the check so that
        # the constructor is only used to create relevant processes.
        with open(tempfile_name, 'w') as handle:
            json.dump({"check_subdomains": False, "bind_address": False}, handle)

        # The `logger` module from the wptserver package uses a singleton
        # pattern which resists testing. In order to avoid conflicting with
        # other tests which rely on that module, pre-existing state is
        # discarded through an explicit "reload" operation.
        reload(logger)

        serve.run(config_path=tempfile_name)

    thread = threading.Thread(target=target)

    thread.start()

    server_subprocesses.get(True, timeout)
    subprocess = server_subprocesses.get(True, timeout)
    subprocess.stop()

    thread.join(timeout)

    assert not thread.is_alive()
