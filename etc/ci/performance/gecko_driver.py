#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from selenium import webdriver
from selenium.common.exceptions import TimeoutException


def execute_gecko_test(testcase, timeout):
    firefox_binary = "./firefox/firefox/firefox"
    driver = webdriver.Firefox(firefox_binary=firefox_binary)
    driver.set_page_load_timeout(timeout)
    failed = False
    try:
        driver.get(testcase)
    except TimeoutException:
        print("Timeout!")
        driver.quit()
        failed = True

    timings = {
        "testcase": testcase,
        "title": driver.title.replace(",", "&#44;")
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

    # We could have just run
    #     timings = driver.execute_script("return performance.timing")
    # But for some test case the webdriver will fail with "too much recursion"
    # So we need to get the timing fields one by one
    for name in timing_names:
        if failed:
            if name == "navigationStart":
                timings[name] = 0
            else:
                timings[name] = -1
        else:
            timings[name] = driver.execute_script(
                "return performance.timing.{0}".format(name)
            )

    timings['testcase'] = testcase
    # driver.quit() gives an "'NoneType' object has no attribute 'path'" error.
    # Fixed in
    # https://github.com/SeleniumHQ/selenium/commit/9157c7071f9900c2608f5ca40ae4f518ed373b96
    driver.quit()
    return [timings]

if __name__ == '__main__':
    # Just for manual testing
    from pprint import pprint
    url = "http://localhost:8000/page_load_test/tp5n/dailymail.co.uk/www.dailymail.co.uk/ushome/index.html"
    pprint(execute_gecko_test(url, 15))
