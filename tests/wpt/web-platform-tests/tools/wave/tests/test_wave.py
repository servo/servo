# mypy: allow-untyped-defs

import errno
import os
import socket
import subprocess
import time

from urllib.request import urlopen
from urllib.error import URLError

from tools.wpt import wpt

def is_port_8080_in_use():
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        s.bind(("127.0.0.1", 8080))
    except OSError as e:
        if e.errno == errno.EADDRINUSE:
            return True
        else:
            raise e
    finally:
        s.close()
    return False

def test_serve():
    if is_port_8080_in_use():
        assert False, "WAVE Test Runner failed: Port 8080 already in use."

    p = subprocess.Popen([os.path.join(wpt.localpaths.repo_root, "wpt"),
        "serve-wave",
        "--config",
        os.path.join(wpt.localpaths.repo_root, "tools/wave/tests/config.json")],
        preexec_fn=os.setsid)

    start = time.time()
    try:
        while True:
            if p.poll() is not None:
                assert False, "WAVE Test Runner failed: Server not running."
            if time.time() - start > 60:
                assert False, "WAVE Test Runner failed: Server did not start responding within 60s."
            try:
                resp = urlopen("http://web-platform.test:8080/_wave/api/sessions/public")
                print(resp)
            except URLError:
                print("URLError")
                time.sleep(1)
            else:
                assert resp.code == 200
                break
    finally:
        os.killpg(p.pid, 15)
