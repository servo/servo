#!/usr/bin/env python3

# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
import shutil
import pathlib
import subprocess
import sys
import time
from decimal import Decimal

from hdc_py.hdc import HarmonyDeviceConnector, HarmonyDevicePerfMode
from selenium import webdriver
from selenium.webdriver.common.options import ArgOptions
from urllib3.exceptions import ProtocolError
from PIL import Image
from selenium.webdriver.remote.webelement import WebElement

WEBDRIVER_PORT = 7000
SERVO_URL = f"http://127.0.0.1:{WEBDRIVER_PORT}"
ABOUT_BLANK = "about:blank"


def calculate_frame_rate():
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
    check_list = []
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
    driver = None
    start_time = time.time()
    while driver is None and time.time() - start_time < timeout:
        try:
            driver = webdriver.Remote(command_executor=SERVO_URL, options=options)
        except (ConnectionError, ProtocolError):
            time.sleep(0.2)
        except Exception as e:
            print(f"Unexpected exception when creating webdriver: {e}, {type(e)}")
            time.sleep(1)
    print(
        f"Established Webdriver connection in {time.time() - start_time}s",
    )
    return driver


def setup_hdc_forward(timeout: int = 5):
    """
    set hdc forward
    :return: If successful, return driver; If failed, return False
    """
    for v in ("HTTP_PROXY", "http_proxy", "HTTPS_PROXY", "https_proxy"):
        os.environ.pop(v, None)
    cmd = ["hdc", "fport", "ls"]
    output = subprocess.check_output(cmd, encoding="utf-8")
    if f"tcp:{WEBDRIVER_PORT} tcp:7000" in output:
        print("HDC port forwarding already established - skipping")
        return

    start_time = time.time()
    while time.time() - start_time < timeout:
        try:
            cmd = ["hdc", "fport", f"tcp:{WEBDRIVER_PORT}", "tcp:7000"]
            print(f"Setting up HDC port forwarding: {' '.join(cmd)}")
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
            # The port forwarding can fail if servo didn't start yet.
            if result.stdout.startswith("[Fail]TCP Port listen failed"):
                time.sleep(0.2)
                continue
            elif result.stdout.startswith("[Fail]"):
                raise RuntimeError(f"HDC port forwarding failed with: {result.stdout}")
            print(f"HDC port forwarding established on port {WEBDRIVER_PORT}")
            return
        except FileNotFoundError:
            print("HDC command not found. Make sure OHOS SDK is installed and hdc is in PATH.")
            raise
        except subprocess.TimeoutExpired:
            print(f"HDC port forwarding timed out on port {WEBDRIVER_PORT}")
            raise
        except Exception as e:
            print(f"failed to setup HDC forwarding: {e}")
            raise
    raise TimeoutError("HDC port forwarding timed out")


def stop_servo():
    """stop servo application"""
    print("Prepare to stop Test Application...")
    cmd = ["hdc", "shell", "aa force-stop org.servo.servo"]
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    print("Stop Test Application successful!")


def element_scroll_into_view_and_rect(driver: webdriver.Remote, element: WebElement):
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


def element_screenshot(element: WebElement, filename: str):
    if not (filename.lower().endswith(".jpg") or filename.lower().endswith(".jpeg")):
        raise ValueError(f"Invalid file type: {filename}. Expected a .jpg/.jpeg file.")

    try:
        print(f"Scrolling {element}")
        region = element_scroll_into_view_and_rect(element)
        time.sleep(2)
        hdc = HarmonyDeviceConnector()
        hdc.screenshot(filename)

        region = Image.open(filename).crop(region)
        save_path = filename + ".png"
        region.save(save_path)
    except Exception as e:
        print(f"Element Screenshot failed with error: {e}")


# We always load "about:blank" first, and then use
# WebDriver to load target url so that it is blocked until fully loaded.
def run_test(test_fn, test_name: str):
    try:
        print("Stopping potential old servo instance ...")
        stop_servo()
        hdc = HarmonyDeviceConnector()
        print("Starting new servo instance...")
        hdc.cmd(f"aa start -a EntryAbility -b org.servo.servo -U {ABOUT_BLANK} --psn --webdriver", timeout=10)
        setup_hdc_forward()
    except Exception as e:
        print(f"Scenario test setup failed with error: {e} (exception: {type(e)})")
        stop_servo()
        sys.exit(1)
    try:
        with HarmonyDevicePerfMode():
            test_fn()
    except Exception as e:
        print(f"Scenario test `{test_name}` failed with error: {e} (exception: {type(e)})")
        hdc.screenshot(f"servo_scenario_{test_name}_error.jpg")
        stop_servo()
        sys.exit(1)
    print("\033[32mTest Succeeded.\033[0m")
    stop_servo()
