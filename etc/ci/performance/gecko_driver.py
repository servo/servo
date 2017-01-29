#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from contextlib import contextmanager
import json
import os
from selenium import webdriver
from selenium.common.exceptions import TimeoutException
import sys


@contextmanager
def create_gecko_session():
    try:
        firefox_binary = os.environ['FIREFOX_BIN']
    except KeyError:
        print("+=============================================================+")
        print("| You must set the path to your firefox binary to FIREFOX_BIN |")
        print("+=============================================================+")
        sys.exit()

    driver = webdriver.Firefox(firefox_binary=firefox_binary)
    yield driver
    # driver.quit() gives an "'NoneType' object has no attribute 'path'" error.
    # Fixed in
    # https://github.com/SeleniumHQ/selenium/commit/9157c7071f9900c2608f5ca40ae4f518ed373b96
    driver.quit()


def generate_placeholder(testcase):
        # We need to still include the failed tests, otherwise Treeherder will
        # consider the result to be a new test series, and thus a new graph. So we
        # use a placeholder with values = -1 to make Treeherder happy, and still be
        # able to identify failed tests (successful tests have time >=0).

        timings = {
            "testcase": testcase,
            "title": ""
        }

        timing_names = [
            "navigationStart",
            "unloadEventStart",
            "domLoading",
            "fetchStart",
            "responseStart",
            "loadEventEnd",
            "connectStart",
            "domainLookupStart",
            "redirectStart",
            "domContentLoadedEventEnd",
            "requestStart",
            "secureConnectionStart",
            "connectEnd",
            "loadEventStart",
            "domInteractive",
            "domContentLoadedEventStart",
            "redirectEnd",
            "domainLookupEnd",
            "unloadEventEnd",
            "responseEnd",
            "domComplete",
        ]

        for name in timing_names:
            timings[name] = 0 if name == "navigationStart" else -1

        return [timings]


def run_gecko_test(testcase, timeout):
    with create_gecko_session() as driver:
        driver.set_page_load_timeout(timeout)
        try:
            driver.get(testcase)
        except TimeoutException:
            print("Timeout!")
            return generate_placeholder(testcase)

        try:
            timings = {
                "testcase": testcase,
                "title": driver.title.replace(",", "&#44;")
            }

            timings.update(json.loads(
                driver.execute_script(
                    "return JSON.stringify(performance.timing)"
                )
            ))
        except:
            # We need to return a timing object no matter what happened.
            # See the comment in generate_placeholder() for explanation
            print("Failed to get a valid timing measurement.")
            return generate_placeholder(testcase)

    return [timings]


if __name__ == '__main__':
    # Just for manual testing
    from pprint import pprint
    url = "http://localhost:8000/page_load_test/tp5n/dailymail.co.uk/www.dailymail.co.uk/ushome/index.html"
    pprint(run_gecko_test(url, 15))
