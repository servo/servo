import sys
from os.path import join, dirname

import mock
import pytest

sys.path.insert(0, join(dirname(__file__), "..", "..", ".."))

sauce = pytest.importorskip("wptrunner.browsers.sauce")

from wptserve.config import Config


def test_sauceconnect_success():
    with mock.patch.object(sauce.SauceConnect, "upload_prerun_exec"),\
            mock.patch.object(sauce.subprocess, "Popen") as Popen,\
            mock.patch.object(sauce.os.path, "exists") as exists:
        # Act as if it's still running
        Popen.return_value.poll.return_value = None
        Popen.return_value.returncode = None
        # Act as if file created
        exists.return_value = True

        sauce_connect = sauce.SauceConnect(
            sauce_user="aaa",
            sauce_key="bbb",
            sauce_tunnel_id="ccc",
            sauce_connect_binary="ddd")

        env_config = Config(browser_host="example.net")
        sauce_connect(None, env_config)
        with sauce_connect:
            pass


@pytest.mark.parametrize("readyfile,returncode", [
    (True, 0),
    (True, 1),
    (True, 2),
    (False, 0),
    (False, 1),
    (False, 2),
])
def test_sauceconnect_failure_exit(readyfile, returncode):
    with mock.patch.object(sauce.SauceConnect, "upload_prerun_exec"),\
            mock.patch.object(sauce.subprocess, "Popen") as Popen,\
            mock.patch.object(sauce.os.path, "exists") as exists,\
            mock.patch.object(sauce.time, "sleep") as sleep:
        Popen.return_value.poll.return_value = returncode
        Popen.return_value.returncode = returncode
        exists.return_value = readyfile

        sauce_connect = sauce.SauceConnect(
            sauce_user="aaa",
            sauce_key="bbb",
            sauce_tunnel_id="ccc",
            sauce_connect_binary="ddd")

        env_config = Config(browser_host="example.net")
        sauce_connect(None, env_config)
        with pytest.raises(sauce.SauceException):
            with sauce_connect:
                pass

        # Given we appear to exit immediately with these mocks, sleep shouldn't be called
        sleep.assert_not_called()


def test_sauceconnect_failure_never_ready():
    with mock.patch.object(sauce.SauceConnect, "upload_prerun_exec"),\
            mock.patch.object(sauce.subprocess, "Popen") as Popen,\
            mock.patch.object(sauce.os.path, "exists") as exists,\
            mock.patch.object(sauce.time, "sleep") as sleep:
        Popen.return_value.poll.return_value = None
        Popen.return_value.returncode = None
        exists.return_value = False

        sauce_connect = sauce.SauceConnect(
            sauce_user="aaa",
            sauce_key="bbb",
            sauce_tunnel_id="ccc",
            sauce_connect_binary="ddd")

        env_config = Config(browser_host="example.net")
        sauce_connect(None, env_config)
        with pytest.raises(sauce.SauceException):
            with sauce_connect:
                pass

        # We should sleep while waiting for it to create the readyfile
        sleep.assert_called()

        # Check we actually kill it after termination fails
        Popen.return_value.terminate.assert_called()
        Popen.return_value.kill.assert_called()


def test_sauceconnect_tunnel_domains():
    with mock.patch.object(sauce.SauceConnect, "upload_prerun_exec"),\
            mock.patch.object(sauce.subprocess, "Popen") as Popen,\
            mock.patch.object(sauce.os.path, "exists") as exists:
        Popen.return_value.poll.return_value = None
        Popen.return_value.returncode = None
        exists.return_value = True

        sauce_connect = sauce.SauceConnect(
            sauce_user="aaa",
            sauce_key="bbb",
            sauce_tunnel_id="ccc",
            sauce_connect_binary="ddd")

        env_config = Config(browser_host="example.net",
                            subdomains={"a", "b"},
                            not_subdomains={"x", "y"})
        sauce_connect(None, env_config)
        with sauce_connect:
            Popen.assert_called_once()
            args, kwargs = Popen.call_args
            cmd = args[0]
            assert "--tunnel-domains" in cmd
            i = cmd.index("--tunnel-domains")
            rest = cmd[i+1:]
            assert len(rest) >= 1
            if len(rest) > 1:
                assert rest[1].startswith("-"), "--tunnel-domains takes a comma separated list (not a space separated list)"
            assert set(rest[0].split(",")) == {'example.net',
                                               'a.example.net',
                                               'b.example.net'}
