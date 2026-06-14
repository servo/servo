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
import os
import pathlib
import random
import shutil
import subprocess
import sys
import time
from contextlib import nullcontext
from dataclasses import dataclass
from decimal import Decimal
from enum import Enum
from types import TracebackType
from typing import TYPE_CHECKING, Callable, Optional, Self, Type

from hdc_py.hdc import HarmonyDeviceConnector, HarmonyDevicePerfMode
from PIL import Image
from selenium import webdriver
from selenium.webdriver.common.options import ArgOptions
from selenium.webdriver.remote.webelement import WebElement
from urllib3.exceptions import MaxRetryError, NewConnectionError, ProtocolError

WEBDRIVER_PORT = random.randrange(9001, 9900)
MITMPROXY_PORT = random.randrange(7150, 9000)
SERVO_URL = f"http://127.0.0.1:{WEBDRIVER_PORT}"
ABOUT_BLANK = "about:blank"
MITMPROXY_VERSION = "12.2.1"
DEFAULT_SERVO_BIN_PATH = "./target/release/servoshell"
SERVO_PACKAGE_NAME = "org.servo.servo"
SCENARIO_TARGET_OS_ENV_VAR = "SERVO_SCENARIO_TARGET_OS"

if TYPE_CHECKING:
    from memory_usage_plotter import MemoryLoggingOptions


class HostOptions(str, Enum):
    LINUX = "linux"
    OHOS = "ohos"
    MACOS = "macos"

    @classmethod
    def from_str(cls, value: str) -> "HostOptions":
        for option in cls:
            if option.value == value:
                return option
        raise ValueError(f"Unsupported host option: {value}")


def get_target_os_from_environment() -> HostOptions:
    return HostOptions.from_str(os.environ.get(SCENARIO_TARGET_OS_ENV_VAR, HostOptions.OHOS.value))


def common_args_from_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument(
        "--target-os",
        type=HostOptions.from_str,
        choices=tuple(option.value for option in HostOptions),
        default=HostOptions.OHOS,
    )
    parser.add_argument(
        "--servo-bin",
        default=DEFAULT_SERVO_BIN_PATH,
        help="Path to the servoshell binary used for local runs",
    )
    args, _ = parser.parse_known_args()
    return args


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


@dataclass
class RunTestOptions:
    test_fn: Callable[[], None]
    test_name: str
    target_os: HostOptions = HostOptions.OHOS
    use_mitmproxy: MitmProxyRunType = MitmProxyRunType.NOPROXY
    dump_file: pathlib.Path = pathlib.Path("/tmp/mitmproxy-dump")
    memory_logging_options: Optional["MemoryLoggingOptions"] = None
    session_history_max_length: int | None = None
    url: str = ABOUT_BLANK
    resolved_servo_bin_path: str = DEFAULT_SERVO_BIN_PATH


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
        exception_type: Type[BaseException] | None,
        exception_value: BaseException | None,
        exception_traceback: TracebackType | None,
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


def create_driver(timeout: int = 10) -> webdriver.Remote:
    print("Trying to create driver")
    options = ArgOptions()
    options.set_capability("browserName", "servo")
    driver: Optional[webdriver.Remote] = None
    start_time = time.time()
    while driver is None and time.time() - start_time < timeout:
        try:
            driver = webdriver.Remote(command_executor=SERVO_URL, options=options)
        except (ConnectionError, MaxRetryError, NewConnectionError, ProtocolError):
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
    timeout: int = 5,
    webdriver_port: int = WEBDRIVER_PORT,
    host_service_port: int = MITMPROXY_PORT,
    target_os: HostOptions = HostOptions.OHOS,
) -> None:
    """
    set hdc forward
    :return: If successful, return driver; If failed, return False
    """
    if target_os != HostOptions.OHOS:
        return

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


def start_servo(
    options: RunTestOptions,
) -> subprocess.Popen[bytes] | None:
    if options.target_os == HostOptions.OHOS:
        hdc = HarmonyDeviceConnector()
        cmd_str = (
            f"aa start -a EntryAbility -b {SERVO_PACKAGE_NAME} -U {options.url} --psn=--webdriver={WEBDRIVER_PORT}"
        )
        if options.use_mitmproxy.should_servo_proxy():
            cmd_str += (
                f" --psn=--pref=network_https_proxy_uri=http://127.0.0.1:{MITMPROXY_PORT}"
                f" --psn=--pref=network_http_proxy_uri=http://127.0.0.1:{MITMPROXY_PORT}"
                " --psn=--ignore-certificate-errors"
            )
        if options.session_history_max_length is not None:
            cmd_str += f" --psn=--pref=session_history_max_length={options.session_history_max_length}"
        hdc.cmd(cmd_str, timeout=10)
        return None

    command = [options.resolved_servo_bin_path, f"--webdriver={WEBDRIVER_PORT}"]
    if options.use_mitmproxy.should_servo_proxy():
        command.extend(
            [
                f"--pref=network_https_proxy_uri=http://127.0.0.1:{MITMPROXY_PORT}",
                f"--pref=network_http_proxy_uri=http://127.0.0.1:{MITMPROXY_PORT}",
                "--ignore-certificate-errors",
            ]
        )
    if options.session_history_max_length is not None:
        command.append(f"--pref=session_history_max_length={options.session_history_max_length}")
    command.append(options.url)
    return subprocess.Popen(command)


def stop_servo(target_os: HostOptions, servo_process: subprocess.Popen[bytes] | None = None) -> None:
    """stop servo application"""
    print("Prepare to stop Test Application...")
    if target_os == HostOptions.OHOS:
        cmd = ["hdc", "shell", f"aa force-stop {SERVO_PACKAGE_NAME}"]
        subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    elif servo_process is not None:
        servo_process.terminate()
        try:
            servo_process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            servo_process.kill()
            servo_process.wait(timeout=10)
    else:
        print("No tracked local Servo process to stop; skipping cleanup.")
    print("Stop Test Application successful!")


def element_scroll_into_view_and_rect(
    driver: webdriver.Remote, element: WebElement
) -> tuple[float, float, float, float]:
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

    if not isinstance(physical_rect, list) or len(physical_rect) != 4:
        raise TypeError("Expected a 4-item rect list")
    left, top, right, bottom = physical_rect
    return (left, top, right, bottom)


def element_screenshot(driver: webdriver.Remote, element: WebElement, filename: str) -> None:
    if not (filename.lower().endswith(".jpg") or filename.lower().endswith(".jpeg")):
        raise ValueError(f"Invalid file type: {filename}. Expected a .jpg/.jpeg file.")

    try:
        print(f"Scrolling {element}")
        region = element_scroll_into_view_and_rect(driver, element)
        time.sleep(2)
        hdc = HarmonyDeviceConnector()
        hdc.screenshot(filename)

        region = Image.open(filename).crop(region)
        save_path = filename + ".png"
        region.save(save_path)
    except Exception as e:
        print(f"Element Screenshot failed with error: {e}")


def is_servo_window_focused(hdc: HarmonyDeviceConnector) -> bool:
    completed_process = hdc.cmd("hidumper -s WindowManagerService -a '-a'", capture_output=True, encoding="utf-8")
    output = str(completed_process.stdout)
    lines = output.splitlines()
    focused_window: Optional[int] = None
    servo_window_id: Optional[int] = None
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


# We always load "about:blank" first, and then use
# WebDriver to load target url so that it is blocked until fully loaded.
def run_test(
    options_or_test_fn: RunTestOptions | Callable[[], None],
    test_name: str | None = None,
    use_mitmproxy: MitmProxyRunType = MitmProxyRunType.NOPROXY,
    session_history_max_length: int | None = None,
    url: Optional[str] = None,
) -> None:
    if isinstance(options_or_test_fn, RunTestOptions):
        options = options_or_test_fn
    else:
        if test_name is None:
            raise ValueError("test_name is required")
        args = common_args_from_args()
        options = RunTestOptions(
            test_fn=options_or_test_fn,
            test_name=test_name,
            target_os=args.target_os,
            use_mitmproxy=use_mitmproxy,
            session_history_max_length=session_history_max_length,
            url=url if url is not None else ABOUT_BLANK,
            resolved_servo_bin_path=args.servo_bin,
        )

    if options.memory_logging_options is not None:
        options.memory_logging_options.target_os = options.target_os
    if os.environ.get("CI") and options.use_mitmproxy == MitmProxyRunType.NOPROXY:
        # if we are in CI and nobody overrode our mitmproxy type we want to replay.
        print("Setting mitmproxy replay")
        options.use_mitmproxy = MitmProxyRunType.REPLAY

    if options.use_mitmproxy == MitmProxyRunType.REPLAY and not options.dump_file.is_file():
        print(f"Dump file {options.dump_file} did not exist. We will abort")
        return
    hdc = HarmonyDeviceConnector() if options.target_os == HostOptions.OHOS else None
    servo_process: subprocess.Popen[bytes] | None = None
    os.environ[SCENARIO_TARGET_OS_ENV_VAR] = options.target_os.value
    try:
        print("Stopping potential old servo instance ...")
        stop_servo(options.target_os)

        setup_hdc_forward(target_os=options.target_os)
        with MitmProxy(options.use_mitmproxy, options.dump_file, MITMPROXY_PORT):
            print("Starting new servo instance...")
            time.sleep(5)
            servo_process = start_servo(options)
            with HarmonyDevicePerfMode() if options.target_os == HostOptions.OHOS else nullcontext():
                if options.target_os == HostOptions.OHOS:
                    assert hdc is not None
                    close_usb_popup(hdc)
                options.test_fn()
    except Exception as e:
        print(f"Scenario test `{options.test_name}` failed with error: {e} (exception: {type(e)})")
        if options.target_os == HostOptions.OHOS:
            assert hdc is not None
            hdc.screenshot(f"servo_scenario_{options.test_name}_error.jpg")
        stop_servo(options.target_os, servo_process)
        sys.exit(1)
    finally:
        os.environ.pop(SCENARIO_TARGET_OS_ENV_VAR, None)
    print("\033[32mTest Succeeded.\033[0m")
    stop_servo(options.target_os, servo_process)
