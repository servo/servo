#!/usr/bin/env python3

# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import json
import sys
import time
import subprocess
from typing import Any
import common_function_for_servo_test
from selenium.webdriver.common.by import By
from selenium.common.exceptions import NoSuchElementException

from python.servo.util import HarmonyDevicePerfMode


def speedometer_to_bmf(speedometer: dict[str, Any], bmf_output: str, profile: str | None = None) -> None:
    output = dict()
    profile = "" if profile is None else profile + "/"

    def parse_speedometer_result(result: dict[str, Any]) -> None:
        if result["unit"] == "ms":
            output[profile + f"Speedometer/{result['name']}"] = {
                "latency": {  # speedometer has ms we need to convert to ns
                    "value": float(result["mean"]) * 1000000.0,
                    "lower_value": float(result["min"]) * 1000000.0,
                    "upper_value": float(result["max"]) * 1000000.0,
                }
            }
        elif result["unit"] == "score":
            output[profile + f"Speedometer/{result['name']}"] = {
                "score": {
                    "value": float(result["mean"]),
                    "lower_value": float(result["min"]),
                    "upper_value": float(result["max"]),
                }
            }
        else:
            raise Exception("Unknown unit!")

        for child in result["children"]:
            parse_speedometer_result(child)

    for v in speedometer.values():
        parse_speedometer_result(v)
    with open(bmf_output, "w", encoding="utf-8") as f:
        json.dump(output, f, indent=4)


def run():
    subprocess.run(
        ["hdc", "shell", "aa", "force-stop", "org.servo.servo"],
        capture_output=True,
        text=True,
        timeout=10,
    )
    subprocess.run(
        [
            "hdc",
            "shell",
            "aa",
            "start",
            "-a",
            "EntryAbility",
            "-b",
            "org.servo.servo",
            "-U",
            "https://servospeedometer.netlify.app",
            "--psn",
            "--webdriver",
        ],
        capture_output=True,
        text=True,
        timeout=1,
    )

    time.sleep(5)

    common_function_for_servo_test.setup_hdc_forward()
    with HarmonyDevicePerfMode():
        driver = common_function_for_servo_test.create_driver()
        if driver is not None:
            try:
                start_button = driver.find_element(By.CLASS_NAME, "start-tests-button")
                time.sleep(10)
                print("Clicking start button")
                start_button.click()
                print("Waiting for speedometer run to finish")
                for i in range(10):
                    time.sleep(30)
                    finished = driver.execute_script("return globalThis.benchmarkClient._hasResults")
                    if finished:
                        break
                print("Getting benchmark result")
                result = driver.execute_script("return globalThis.benchmarkClient._formattedJSONResult({modern: true})")
                speedometer_json = json.loads(result)

                print("Writing to file")
                try:
                    speedometer_to_bmf(speedometer_json, "speedometer.json", sys.argv[1])
                    return True
                except IndexError:
                    print("You need to supply a profile")
                    return False
            except json.decoder.JSONDecodeError:
                print("Error: Failed to parse speedometer results")
                return False
            except NoSuchElementException:
                print("Could not find element, we probably did not load the page.")
                return False
            except Exception as e:  # noqa: E722
                print(f"Exception caught: {e}")
                return False


if __name__ == "__main__":
    result = run()
    if not result:
        sys.exit(1)
