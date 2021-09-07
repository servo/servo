import errno
import os
import socket
import subprocess
import time

from urllib.request import urlopen
from urllib.error import URLError

import pytest

from tools.wpt import wpt

def is_port_8000_in_use():
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        s.bind(("127.0.0.1", 8000))
    except OSError as e:
        if e.errno == errno.EADDRINUSE:
            return True
        else:
            raise e
    finally:
        s.close()
    return False

def test_serve():
    if is_port_8000_in_use():
        pytest.skip("WAVE Test Runner failed: Port 8000 already in use.")

    p = subprocess.Popen([os.path.join(wpt.localpaths.repo_root, "wpt"), "serve-wave"],
                         preexec_fn=os.setsid)

    start = time.time()
    try:
        while True:
            if p.poll() is not None:
                assert False, "WAVE Test Runner failed: Server not running."
            if time.time() - start > 6 * 60:
                assert False, "WAVE Test Runner failed: Server did not start responding within 6m."
            try:
                resp = urlopen("http://web-platform.test:8000/_wave/api/sessions/public")
                print(resp)
            except URLError:
                print("Server not responding, waiting another 10s.")
                time.sleep(10)
            else:
                assert resp.code == 200
                break
    finally:
        os.killpg(p.pid, 15)
