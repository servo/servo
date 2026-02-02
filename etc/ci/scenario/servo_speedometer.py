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
from typing import Any
import common_function_for_servo_test
from selenium.webdriver.common.by import By
from selenium.common.exceptions import NoSuchElementException


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


def run_speedometer():
    driver = common_function_for_servo_test.create_driver()
    try:
        driver.get("https://servospeedometer.netlify.app")
        driver.implicitly_wait(10)
        start_button = driver.find_element(By.CLASS_NAME, "start-tests-button")
    except NoSuchElementException:
        print("Error: Could not find the start button. There might be a network issue, or the page changed")
        raise
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
    try:
        speedometer_json = json.loads(result)
    except json.decoder.JSONDecodeError:
        print("Error: Failed to parse speedometer results")
        print(f"json result: `{result}`")
        raise

    print("Writing to file")
    try:
        speedometer_to_bmf(speedometer_json, "speedometer.json", sys.argv[1])
    except IndexError:
        # this will be caught by the main exec so we are probably in a cache run.
        print("You need to supply a profile. Not storing data.")


if __name__ == "__main__":
    # todo: use argparse
    try:
        sys.argv[1]
    except IndexError:
        print("Usage: You need to supply a profile for the bencher bmf output")
        sys.exit(1)
    common_function_for_servo_test.run_test(run_speedometer, "speedometer")
