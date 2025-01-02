#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import submit_to_perfherder


def test_format_testcase_name():
    assert ('about:blank' == submit_to_perfherder.format_testcase_name(
        'about:blank'))
    assert ('163.com' == submit_to_perfherder.format_testcase_name((
        'http://localhost:8000/page_load_test/163.com/p.mail.163.com/'
        'mailinfo/shownewmsg_www_1222.htm.html')))
    assert (('1234567890223456789032345678904234567890'
            '5234567890623456789072345678908234567890')
            == submit_to_perfherder.format_testcase_name((
                '1234567890223456789032345678904234567890'
                '52345678906234567890723456789082345678909234567890')))
    assert ('news.ycombinator.com' == submit_to_perfherder.format_testcase_name(
        'http://localhost:8000/tp6/news.ycombinator.com/index.html'))


def test_format_perf_data():
    mock_result = [
        {
            "unloadEventStart": None,
            "domLoading": 1460444930000,
            "fetchStart": None,
            "responseStart": None,
            "loadEventEnd": None,
            "connectStart": None,
            "domainLookupStart": None,
            "redirectStart": None,
            "domContentLoadedEventEnd": 1460444930000,
            "requestStart": None,
            "secureConnectionStart": None,
            "connectEnd": None,
            "navigationStart": 1460444930000,
            "loadEventStart": None,
            "domInteractive": 1460444930000,
            "domContentLoadedEventStart": 1460444930000,
            "redirectEnd": None,
            "domainLookupEnd": None,
            "unloadEventEnd": None,
            "responseEnd": None,
            "testcase": "about:blank",
            "domComplete": 1460444931000
        },
        {
            "unloadEventStart": None,
            "domLoading": 1460444934000,
            "fetchStart": None,
            "responseStart": None,
            "loadEventEnd": None,
            "connectStart": None,
            "domainLookupStart": None,
            "redirectStart": None,
            "domContentLoadedEventEnd": 1460444946000,
            "requestStart": None,
            "secureConnectionStart": None,
            "connectEnd": None,
            "navigationStart": 1460444934000,
            "loadEventStart": None,
            "domInteractive": 1460444946000,
            "domContentLoadedEventStart": 1460444946000,
            "redirectEnd": None,
            "domainLookupEnd": None,
            "unloadEventEnd": None,
            "responseEnd": None,
            "testcase": ("http://localhost:8000/page_load_test/163.com/"
                         "p.mail.163.com/mailinfo/"
                         "shownewmsg_www_1222.htm.html"),
            "domComplete": 1460444948000
        }
    ]

    expected = {
        "performance_data": {
            "framework": {"name": "servo-perf"},
            "suites": [
                {
                    "name": "domComplete",
                    "value": 3741.657386773941,
                    "subtests": [
                        {"name": "about:blank",
                         "value": 1000},
                        {"name": "163.com",
                         "value": 14000},
                    ]
                }
            ]
        }
    }
    result = submit_to_perfherder.format_perf_data(mock_result)
    assert (expected == result)


def test_format_bad_perf_data():
    mock_result = [
        {
            "navigationStart": 1460444930000,
            "testcase": "about:blank",
            "domComplete": 0
        },
        {
            "navigationStart": 1460444934000,
            "testcase": ("http://localhost:8000/page_load_test/163.com/"
                         "p.mail.163.com/mailinfo/"
                         "shownewmsg_www_1222.htm.html"),
            "domComplete": 1460444948000
        }
    ]

    expected = {
        "performance_data": {
            "framework": {"name": "servo-perf"},
            "suites": [
                {
                    "name": "domComplete",
                    "value": 14000.0,
                    "subtests": [
                        {"name": "about:blank",
                         "value": -1},  # Timeout
                        {"name": "163.com",
                         "value": 14000},
                    ]
                }
            ]
        }
    }
    result = submit_to_perfherder.format_perf_data(mock_result)
    assert (expected == result)
