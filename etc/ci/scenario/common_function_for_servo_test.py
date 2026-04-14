#!/usr/bin/env python3

# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import argparse
import enum
import inspect
import os
import pathlib
import random
import shutil
import subprocess
import sys
import time
from decimal import Decimal
from enum import Enum
from types import TracebackType
from typing import Any, Callable, Optional, Self, Type

from hdc_py.hdc import HarmonyDeviceConnector, HarmonyDevicePerfMode  # type: ignore[import-untyped]
from selenium import webdriver
from selenium.webdriver.common.options import ArgOptions
from selenium.webdriver.remote.webelement import WebElement
from urllib3.exceptions import ProtocolError
from memory_usage_plotter import MemoryLoggingOptions, NonBlockingMemoryLogging, HostOptions

WEBDRIVER_PORT = random.randrange(9001, 9999)
MITMPROXY_PORT = random.randrange(7150, 9000)
SERVO_URL = f"http://127.0.0.1:{WEBDRIVER_PORT}"
ABOUT_BLANK = "about:blank"
MITMPROXY_VERSION = "12.2.1"
SERVO_BIN_PATH = "./target/release/servoshell"


class MitmProxyRunType(enum.Enum):
    # replay a recorded interaction
    REPLAY = (1,)
    # record an interaction
    RECORD = (2,)
    # start proxy and just forward
    FORWARD = (3,)
    # do not do anything with the proxy
    NOPROXY = (4,)

    def should_servo_proxy(self) -> bool:
        return self == MitmProxyRunType.REPLAY or self == MitmProxyRunType.RECORD or self == MitmProxyRunType.FORWARD


class MitmProxy:
    def __init__(self, use_proxy: MitmProxyRunType, dump_file: pathlib.Path, port: int) -> None:
        self.mitmproxy: subprocess.Popen[bytes] | None = None
        self.use_proxy = use_proxy
        self.dump_file = dump_file
        self.port = port

    def __enter__(self) -> Self:
        # for record the external recorder will record
        # make sure mitmproxy is installed
        if self.use_proxy == MitmProxyRunType.REPLAY:
            print("Running mitmproxy for replay")
            self.mitmproxy = subprocess.Popen(
                [
                    "mitmdump",
                    "-p",
                    str(self.port),
                    "--server-replay",
                    self.dump_file,
                    # reply with 404 if request is not in dump_file
                    "--set",
                    "server_replay_extra=404",
                    # do not delete a request from the dump_file after fulfillment
                    "--set",
                    "server_replay_reuse=true",
                ]
            )
        elif self.use_proxy == MitmProxyRunType.FORWARD:
            print("Running mitmproxy in forwarding mode")
            self.mitmproxy = subprocess.Popen(
                [
                    "mitmdump",
                    "-w",
                    self.dump_file,
                    # "--mode", "upstream:http://127.0.0.1:3128",
                    "-p",
                    str(self.port),
                    "--set",
                    "ssl_insecure=true",
                ]
            )
        return self

    def __exit__(
        self,
        exc_type: Type[BaseException] | None,
        exc: BaseException | None,
        tb: TracebackType | None,
    ) -> None:
        if self.mitmproxy:
            print("Killing mitmproxy")
            self.mitmproxy.kill()
            time.sleep(2)


def calculate_frame_rate() -> float:
    """
    Pull trace from device and calculate frame rate through trace
    calculate frame rate: When there are elements moving on the page, H: EndCommands will be printed
    to indicate that the frame is being sent out. After capturing the trace, the frame rate can be obtained
    by calculating the number of frames per second.
    :return: frame rate
    """
    print("Prepare to create local dir to put trace file...")
    target_path = os.path.join(pathlib.Path(os.path.dirname(__file__)).parent.parent.parent, "target")
    os.makedirs(target_path, exist_ok=True)
    ci_testing_path = os.path.join(target_path, "ci_testing")
    os.makedirs(ci_testing_path, exist_ok=True)
    print("Create local dir success.")

    file_name = os.path.join(ci_testing_path, "my_trace.html")
    cmd = ["hdc", "file", "recv", "/data/local/tmp/my_trace.html", f"{file_name}"]
    print("Pulling trace file from device...")
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    print(f"Pull trace file to {file_name} success.")

    trace_key = "H:ReceiveVsync"
    check_list: list[str] = []
    with open(file_name, "r") as f:
        lines = f.readlines()
        if "TouchHandler::FlingStart" not in lines:
            raise RuntimeError("No 'TouchHandler::FlingStart' signals found in the trace file.")
        for line in range(len(lines)):
            if "TouchHandler::FlingStart" in lines[line]:
                check_list = []
            elif "TouchHandler::FlingEnd" in lines[line]:
                break
            else:
                check_list.append(lines[line])
    matching_lines = [
        check_list[line]
        for line in range(len(check_list))
        if (trace_key in check_list[line]) and ("render_service" in check_list[line])
    ]
    if len(matching_lines) == 0:
        raise RuntimeError("No 'H:ReceiveVsync' signals found in the trace file.")
    start_time = matching_lines[0].split()[5].split(":")[0]
    end_time = matching_lines[-1].split()[5].split(":")[0]
    interval_time = Decimal(end_time) - Decimal(start_time)
    shutil.rmtree(target_path)
    framerate = round(float((len(matching_lines) - 1) / interval_time), 2)
    if framerate > 120:
        print(f"Framerate {framerate} is unexpectedly higher than 120")
    return min(framerate, 120.00)


def create_driver(timeout: int = 10, servo_url: Optional[str] = None) -> webdriver.Remote:
    if servo_url is None:
        servo_url = SERVO_URL
    print("Trying to create driver")
    options = ArgOptions()
    options.set_capability("browserName", "servo")
    driver: Optional[webdriver.Remote] = None
    start_time = time.time()
    while driver is None and time.time() - start_time < timeout:
        try:
            driver = webdriver.Remote(command_executor=servo_url, options=options)
        except (ConnectionError, ProtocolError):
            time.sleep(0.2)
        except Exception as e:
            print(f"Unexpected exception when creating webdriver: {e}, {type(e)}")
            time.sleep(1)
    if driver is None:
        raise RuntimeError(f"The driver is not created due to {timeout}s timeout (took: {time.time() - start_time}s)")
    else:
        print(
            f"Established Webdriver connection in {time.time() - start_time}s",
        )
    return driver


class PortMapResult(Enum):
    SUCCESSFUL = (1,)
    PORT_EXISTS = (2,)
    FORWARD_FAILED = (3,)

    def is_success(self) -> bool:
        return self == PortMapResult.SUCCESSFUL or self == PortMapResult.PORT_EXISTS


# Sets up the port forward.
def port_forward(port: int | str, reverse: bool) -> PortMapResult:
    cmd = ["hdc", "fport", "ls"]
    output = subprocess.check_output(cmd, encoding="utf-8")
    if f"tcp:{port}" in output:
        return PortMapResult.PORT_EXISTS

    cmd = []
    if reverse:
        cmd = ["hdc", "rport", f"tcp:{port}", f"tcp:{port}"]
    else:
        cmd = ["hdc", "fport", f"tcp:{port}", f"tcp:{port}"]
    print(f"Setting up HDC port forwarding: {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    if result.stdout.startswith("[Fail]TCP Port listen failed"):
        print("Forward failed")
        return PortMapResult.FORWARD_FAILED
    elif result.stdout.startswith("[Fail]"):
        print("Forward failed other way")
        raise RuntimeError(f"HDC port forwarding failed with: {result.stdout}")

    print("Port forward successful")
    return PortMapResult.SUCCESSFUL


def setup_hdc_forward(
    timeout: int = 5, webdriver_port: int = WEBDRIVER_PORT, host_service_port: int = MITMPROXY_PORT
) -> None:
    """
    set hdc forward
    :return: If successful, return driver; If failed, return False
    """
    for v in ("HTTP_PROXY", "http_proxy", "HTTPS_PROXY", "https_proxy"):
        os.environ.pop(v, None)

    start_time = time.time()
    while time.time() - start_time < timeout:
        try:
            if port_forward(webdriver_port, False).is_success() and port_forward(host_service_port, True).is_success():
                return
            time.sleep(0.2)
        except FileNotFoundError:
            print("HDC command not found. Make sure OHOS SDK is installed and hdc is in PATH.")
            raise
        except subprocess.TimeoutExpired:
            print(f"HDC port forwarding timed out on port {webdriver_port}")
            raise
        except Exception as e:
            print(f"failed to setup HDC forwarding: {e}")
            raise
    raise TimeoutError("HDC port forwarding timed out")


def stop_servo() -> None:
    """stop servo application"""
    print("Prepare to stop Test Application...")
    cmd = ["hdc", "shell", "aa force-stop org.servo.servo"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    print("Stop Test Application successful!")


def element_scroll_into_view_and_rect(driver: webdriver.Remote, element: WebElement) -> Any:
    """
    This scrolls element into view, and return the DOMRect tuple:
    [left, top, right, bottom]
    """
    # false corresponds to `scrollIntoViewOptions: {block: "end", inline: "nearest"}`
    # which is consistent with standard webdriver screenshot.
    # <https://w3c.github.io/webdriver/#dfn-scrolls-into-view>
    driver.execute_script(
        "arguments[0].scrollIntoView(false)",
        element,
    )

    # sleep as animation may be delayed.
    time.sleep(1)

    physical_rect = driver.execute_script(
        """
        const rect = arguments[0].getBoundingClientRect();
        const dpr = window.devicePixelRatio;
        return [
            rect.left * dpr,
            rect.top * dpr,
            rect.right * dpr,
            rect.bottom * dpr
        ];
        """,
        element,
    )

    return physical_rect


def is_servo_window_focused(hdc: HarmonyDeviceConnector) -> bool:
    completed_process = hdc.cmd("hidumper -s WindowManagerService -a '-a'", capture_output=True, encoding="utf-8")
    output = str(completed_process.stdout)
    lines = output.splitlines()
    focused_window = None
    servo_window_id = None
    for line in lines:
        if line.lower().startswith("focus window:"):
            focused_window = int(line.split(":")[1].strip())
        if line.lower().startswith("servo"):
            # The table format is as follows:
            # WindowName DisplayId Pid WinId
            servo_window_id = int(line.split()[3])
        if line.lower().startswith("windowname"):
            table_headers = line.split()
            assert table_headers[0].lower() == "windowname"
            assert table_headers[3].lower() == "winid"
        if focused_window is not None and servo_window_id is not None:
            break
    if focused_window is None or servo_window_id is None:
        raise RuntimeError("Could not find focused window or servo window id.")
    return focused_window == servo_window_id


def close_usb_popup(hdc: HarmonyDeviceConnector) -> None:
    """
    When connecting an OpenHarmony device, a system pop-up will be opened on the device,
    asking the user to confirm which USB mode should be used for the connection.
    This pop-up overlays servo, and will hence disturb some inputs, and affect screenshots.
    """
    try:
        if not is_servo_window_focused(hdc):
            print("The focused window does not belong to servo. Sending back-event to try and close the window.")
            # The USB pop-up can be dismissed by simulating the back key-event.
            hdc.cmd("uitest uiInput keyEvent Back")
        if not is_servo_window_focused(hdc):
            print("The focused window still isn't servo. Giving up.")
    except Exception as e:
        print(f"Internal error trying to close the USB pop-up overlay: {e}. Ignoring...")


def _invoke_test_fn(
    test_fn: Callable,
    driver: Optional[webdriver.Remote] = None,
    memory_logging: Optional[NonBlockingMemoryLogging] = None,
) -> None:
    parameters = inspect.signature(test_fn).parameters
    kwargs: dict[str, Any] = {}
    if driver is not None and "driver" in parameters:
        kwargs["driver"] = driver
    if memory_logging is not None and "memory_logging" in parameters:
        kwargs["memory_logging"] = memory_logging

    test_fn(**kwargs)


def _parse_target_os_arg() -> Optional[HostOptions]:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--target_os", choices=[option.value for option in HostOptions])
    args, _ = parser.parse_known_args()
    if args.target_os is None:
        return None
    return HostOptions(args.target_os)


def _detect_host_target_os() -> HostOptions:
    if sys.platform == "darwin":
        return HostOptions.MACOS
    if sys.platform.startswith("linux"):
        return HostOptions.LINUX
    raise ValueError(f"Unsupported host platform: {sys.platform!r}")


def _resolve_target_os(target_os_arg: Optional[HostOptions]) -> HostOptions:
    if target_os_arg is not None:
        return target_os_arg

    parsed_target_os = _parse_target_os_arg()
    if parsed_target_os is not None:
        return parsed_target_os

    target_os_env = os.environ.get("TARGET_OS")
    if target_os_env:
        try:
            return HostOptions(target_os_env)
        except ValueError as exc:
            raise ValueError(f"Unsupported TARGET_OS value: {target_os_env!r}") from exc

    return _detect_host_target_os()


def _create_memory_logging_options(
    use_memory_logging: Optional[MemoryLoggingOptions] | bool,
    target_os: HostOptions,
    test_name: str,
) -> Optional[MemoryLoggingOptions]:
    if isinstance(use_memory_logging, MemoryLoggingOptions):
        options = use_memory_logging
    elif use_memory_logging or os.environ.get("PLOT"):
        options = MemoryLoggingOptions(log_to_file=True, pre_time=0.2, plot=True)
    else:
        return None

    if os.environ.get("PLOT"):
        options.log_to_file = True
        options.plot = True

    options.host = target_os
    options.test_name = test_name
    return options


def _run_test_with_memory_logging(
    test_fn: Callable,
    memory_logging_options: MemoryLoggingOptions,
    driver_factory: Callable[[], webdriver.Remote],
    driver_setup: Optional[Callable[[webdriver.Remote], None]] = None,
) -> None:
    parameters = inspect.signature(test_fn).parameters
    needs_driver = "driver" in parameters
    needs_shared_driver = needs_driver or bool(getattr(memory_logging_options, "reset_tab", None))

    if needs_shared_driver:
        driver = driver_factory()
        if driver_setup is not None:
            driver_setup(driver)
        with NonBlockingMemoryLogging(options=memory_logging_options, driver=driver) as logger:
            _invoke_test_fn(test_fn, driver=driver, memory_logging=logger)
        return

    with NonBlockingMemoryLogging(options=memory_logging_options) as logger:
        _invoke_test_fn(test_fn, memory_logging=logger)


# We always load "about:blank" first, and then use
# WebDriver to load target url so that it is blocked until fully loaded.
def run_test(
    test_fn: Callable,
    test_name: str,
    target_os_arg: Optional[HostOptions] = None,
    use_mitmproxy: MitmProxyRunType = MitmProxyRunType.NOPROXY,
    use_memory_logging: Optional[MemoryLoggingOptions] | bool = None,
    url: Optional[str] = None,
) -> None:
    parsed_target_os = _parse_target_os_arg()
    target_os = _resolve_target_os(target_os_arg)
    target_os_env = os.environ.get("TARGET_OS")
    if target_os_arg is not None:
        print(f"Using run_test arg ({target_os.value})")
    elif parsed_target_os is not None:
        print(f"Using command line arg ({target_os.value})")
    elif target_os_env:
        print(f"Using env var ({target_os.value})")
    else:
        print(f"Using host OS ({target_os.value})")

    if os.environ.get("CI") and use_mitmproxy == MitmProxyRunType.NOPROXY:
        # if we are in CI and nobody overrode our mitmproxy type we want to replay.
        print("Setting mitmproxy replay")
        use_mitmproxy = MitmProxyRunType.REPLAY

    dump_file = pathlib.Path("/tmp/mitmproxy-dump")
    if use_mitmproxy == MitmProxyRunType.REPLAY and not dump_file.is_file():
        raise FileNotFoundError(f"Dump file {dump_file} does not exist for replay mode")

    memory_logging_options = _create_memory_logging_options(use_memory_logging, target_os, test_name)
    session_history_max_length = 0 if memory_logging_options is not None and memory_logging_options.reset_tab else 20

    url = url if url is not None else ABOUT_BLANK

    if target_os == HostOptions.OHOS:
        hdc = HarmonyDeviceConnector()
        try:
            print("Stopping potential old servo instance ...")
            stop_servo()

            setup_hdc_forward()
            with MitmProxy(use_mitmproxy, dump_file, MITMPROXY_PORT):
                print("Starting new servo instance...")
                time.sleep(5)
                cmd_str = f"aa start -a EntryAbility -b org.servo.servo -U {url} --psn=--webdriver={WEBDRIVER_PORT} --psn=--pref=session_history_max_length={session_history_max_length}"
                if use_mitmproxy.should_servo_proxy():
                    cmd_str += f" --psn=--pref=network_https_proxy_uri=http://127.0.0.1:{str(MITMPROXY_PORT)} --psn=--pref=network_http_proxy_uri=http://127.0.0.1:{MITMPROXY_PORT} --psn=--ignore-certificate-errors "
                hdc.cmd(
                    cmd_str,
                    timeout=10,
                )
                with HarmonyDevicePerfMode():
                    close_usb_popup(hdc)
                    if memory_logging_options is not None:
                        _run_test_with_memory_logging(
                            test_fn,
                            memory_logging_options,
                            lambda: create_driver(timeout=1, servo_url=SERVO_URL),
                        )
                    else:
                        _invoke_test_fn(test_fn)
        except Exception as e:
            print(f"Scenario test `{test_name}` failed with error: {e} (exception: {type(e)})")
            hdc.screenshot(f"servo_scenario_{test_name}_error.jpg")
            sys.exit(1)
        finally:
            stop_servo()
        print("\033[32mTest Succeeded.\033[0m")
    elif target_os == HostOptions.MACOS or target_os == HostOptions.LINUX:
        kill_servo()
        try:
            with MitmProxy(use_mitmproxy, dump_file, MITMPROXY_PORT):
                print("Starting new servo instance...")
                start_servo(
                    WEBDRIVER_PORT,
                    SERVO_BIN_PATH,
                    delay=2,
                    session_history_max_length=session_history_max_length,
                    use_proxy=use_mitmproxy.should_servo_proxy(),
                    mitmproxy_port=MITMPROXY_PORT,
                )
                if memory_logging_options is None:
                    _invoke_test_fn(test_fn)
                else:
                    _run_test_with_memory_logging(
                        test_fn,
                        memory_logging_options,
                        lambda: create_driver(servo_url=f"http://127.0.0.1:{WEBDRIVER_PORT}"),
                        lambda webdriver: webdriver.implicitly_wait(30),
                    )
                print("\033[32mTest Succeeded.\033[0m")
        except Exception as e:
            print(f"Scenario test `{test_name}` failed with error: {e} (exception: {type(e)})")
            sys.exit(1)
        finally:
            kill_servo()
    else:
        raise ValueError(f"Unsupported target OS: {target_os}")


def start_servo(
    webdriver_port: int,
    servo_path: str,
    delay: int = 0,
    session_history_max_length: int = 20,
    use_proxy: bool = False,
    mitmproxy_port: Optional[int] = None,
    url: Optional[str] = None,
) -> None:
    """Start servo and create webdriver"""
    try:
        cmd = [
            servo_path,
            f"--webdriver={webdriver_port}",
            f"--pref=session_history_max_length={session_history_max_length}",
        ]
        if use_proxy and mitmproxy_port is not None:
            cmd.extend(
                [
                    f"--pref=network_https_proxy_uri=http://127.0.0.1:{mitmproxy_port}",
                    f"--pref=network_http_proxy_uri=http://127.0.0.1:{mitmproxy_port}",
                    "--ignore-certificate-errors",
                ]
            )
        cmd.extend([f" {ABOUT_BLANK if url is None else url}"])
        subprocess.Popen(cmd)
    except FileNotFoundError:
        print("The servo binary does not exist")
        sys.exit(1)
    if delay > 0:
        time.sleep(delay)


def kill_servo() -> None:
    subprocess.run(["killall", "servoshell"])
