#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import runner
import pytest


def test_log_parser():
    mock_url = "http://localhost:8000/page_load_test/56.com/www.56.com/index.html"
    mock_log = b'''
[PERF] perf block start
[PERF],testcase,http://localhost:8000/page_load_test/56.com/www.56.com/index.html
[PERF],navigationStart,1460358376
[PERF],unloadEventStart,undefined
[PERF],unloadEventEnd,undefined
[PERF],redirectStart,undefined
[PERF],redirectEnd,undefined
[PERF],fetchStart,undefined
[PERF],domainLookupStart,undefined
[PERF],domainLookupEnd,undefined
[PERF],connectStart,undefined
[PERF],connectEnd,undefined
[PERF],secureConnectionStart,undefined
[PERF],requestStart,undefined
[PERF],responseStart,undefined
[PERF],responseEnd,undefined
[PERF],domLoading,1460358376000
[PERF],domInteractive,1460358388000
[PERF],domContentLoadedEventStart,1460358388000
[PERF],domContentLoadedEventEnd,1460358388000
[PERF],domComplete,1460358389000
[PERF],loadEventStart,undefined
[PERF],loadEventEnd,undefined
[PERF] perf block end
Shutting down the Constellation after generating an output file or exit flag specified
'''

    expected = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "navigationStart": 1460358376,
        "unloadEventStart": None,
        "unloadEventEnd": None,
        "redirectStart": None,
        "redirectEnd": None,
        "fetchStart": None,
        "domainLookupStart": None,
        "domainLookupEnd": None,
        "connectStart": None,
        "connectEnd": None,
        "secureConnectionStart": None,
        "requestStart": None,
        "responseStart": None,
        "responseEnd": None,
        "domLoading": 1460358376000,
        "domInteractive": 1460358388000,
        "domContentLoadedEventStart": 1460358388000,
        "domContentLoadedEventEnd": 1460358388000,
        "domComplete": 1460358389000,
        "loadEventStart": None,
        "loadEventEnd": None
    }]
    result = runner.parse_log(mock_log, mock_url)
    assert (expected == list(result))


def test_log_parser_complex():
    mock_log = b'''
[PERF] perf block start
[PERF],testcase,http://localhost:8000/page_load_test/56.com/www.56.com/content.html
[PERF],navigationStart,1460358300
[PERF],unloadEventStart,undefined
[PERF],unloadEventEnd,undefined
[PERF],redirectStart,undefined
[PERF],redirectEnd,undefined
[PERF],fetchStart,undefined
[PERF],domainLookupStart,undefined
[PERF],domainLookupEnd,undefined
[PERF],connectStart,undefined
[PERF],connectEnd,undefined
[PERF],secureConnectionStart,undefined
[PERF],requestStart,undefined
[PERF],responseStart,undefined
[PERF],responseEnd,undefined
[PERF],domLoading,1460358376000
[PERF],domInteractive,1460358388000
[PERF],domContentLoadedEventStart,1460358388000
[PERF],domContentLoadedEventEnd,1460358388000
[PERF],domComplete,1460358389000
[PERF],loadEventStart,undefined
[PERF],loadEventEnd,undefined
[PERF] perf block end
Some other js error logs here

[PERF] perf block start
[PERF],testcase,http://localhost:8000/page_load_test/56.com/www.56.com/index.html
[PERF],navigationStart,1460358376
[PERF],unloadEventStart,undefined
[PERF],unloadEventEnd,undefined
[PERF],redirectStart,undefined
[PERF],redirectEnd,undefined
[PERF],fetchStart,undefined
[PERF],domainLookupStart,undefined
[PERF],domainLookupEnd,undefined
[PERF],connectStart,undefined
[PERF],connectEnd,undefined
[PERF],secureConnectionStart,undefined
[PERF],requestStart,undefined
[PERF],responseStart,undefined
[PERF],responseEnd,undefined
[PERF],domLoading,1460358376000
[PERF],domInteractive,1460358388000
[PERF],domContentLoadedEventStart,1460358388000
[PERF],domContentLoadedEventEnd,1460358388000
[PERF],domComplete,1460358389000
[PERF],loadEventStart,undefined
[PERF],loadEventEnd,undefined
[PERF] perf block end
Shutting down the Constellation after generating an output file or exit flag specified
'''
    mock_url = "http://localhost:8000/page_load_test/56.com/www.56.com/index.html"
    expected = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "navigationStart": 1460358376,
        "unloadEventStart": None,
        "unloadEventEnd": None,
        "redirectStart": None,
        "redirectEnd": None,
        "fetchStart": None,
        "domainLookupStart": None,
        "domainLookupEnd": None,
        "connectStart": None,
        "connectEnd": None,
        "secureConnectionStart": None,
        "requestStart": None,
        "responseStart": None,
        "responseEnd": None,
        "domLoading": 1460358376000,
        "domInteractive": 1460358388000,
        "domContentLoadedEventStart": 1460358388000,
        "domContentLoadedEventEnd": 1460358388000,
        "domComplete": 1460358389000,
        "loadEventStart": None,
        "loadEventEnd": None
    }]
    result = runner.parse_log(mock_log, mock_url)
    assert (expected == list(result))


def test_log_parser_empty():
    mock_log = b'''
[PERF] perf block start
[PERF]BROKEN!!!!!!!!!1
[PERF]BROKEN!!!!!!!!!1
[PERF]BROKEN!!!!!!!!!1
[PERF]BROKEN!!!!!!!!!1
[PERF]BROKEN!!!!!!!!!1
[PERF] perf block end
'''
    mock_testcase = "http://localhost:8000/page_load_test/56.com/www.56.com/index.html"

    expected = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "title": "",
        "navigationStart": 0,
        "unloadEventStart": -1,
        "unloadEventEnd": -1,
        "redirectStart": -1,
        "redirectEnd": -1,
        "fetchStart": -1,
        "domainLookupStart": -1,
        "domainLookupEnd": -1,
        "connectStart": -1,
        "connectEnd": -1,
        "secureConnectionStart": -1,
        "requestStart": -1,
        "responseStart": -1,
        "responseEnd": -1,
        "domLoading": -1,
        "domInteractive": -1,
        "domContentLoadedEventStart": -1,
        "domContentLoadedEventEnd": -1,
        "domComplete": -1,
        "loadEventStart": -1,
        "loadEventEnd": -1
    }]
    result = runner.parse_log(mock_log, mock_testcase)
    assert (expected == list(result))


def test_log_parser_error():
    mock_log = b'Nothing here! Test failed!'
    mock_testcase = "http://localhost:8000/page_load_test/56.com/www.56.com/index.html"

    expected = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "title": "",
        "navigationStart": 0,
        "unloadEventStart": -1,
        "unloadEventEnd": -1,
        "redirectStart": -1,
        "redirectEnd": -1,
        "fetchStart": -1,
        "domainLookupStart": -1,
        "domainLookupEnd": -1,
        "connectStart": -1,
        "connectEnd": -1,
        "secureConnectionStart": -1,
        "requestStart": -1,
        "responseStart": -1,
        "responseEnd": -1,
        "domLoading": -1,
        "domInteractive": -1,
        "domContentLoadedEventStart": -1,
        "domContentLoadedEventEnd": -1,
        "domComplete": -1,
        "loadEventStart": -1,
        "loadEventEnd": -1
    }]
    result = runner.parse_log(mock_log, mock_testcase)
    assert (expected == list(result))


def test_log_parser_bad_testcase_name():
    mock_testcase = "http://localhost:8000/page_load_test/56.com/www.56.com/index.html"
    # Notice the testcase is about:blank, servo crashed
    mock_log = b'''
[PERF] perf block start
[PERF],testcase,about:blank
[PERF],navigationStart,1460358376
[PERF],unloadEventStart,undefined
[PERF],unloadEventEnd,undefined
[PERF],redirectStart,undefined
[PERF],redirectEnd,undefined
[PERF],fetchStart,undefined
[PERF],domainLookupStart,undefined
[PERF],domainLookupEnd,undefined
[PERF],connectStart,undefined
[PERF],connectEnd,undefined
[PERF],secureConnectionStart,undefined
[PERF],requestStart,undefined
[PERF],responseStart,undefined
[PERF],responseEnd,undefined
[PERF],domLoading,1460358376000
[PERF],domInteractive,1460358388000
[PERF],domContentLoadedEventStart,1460358388000
[PERF],domContentLoadedEventEnd,1460358388000
[PERF],domComplete,1460358389000
[PERF],loadEventStart,undefined
[PERF],loadEventEnd,undefined
[PERF] perf block end
Shutting down the Constellation after generating an output file or exit flag specified
'''

    expected = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "title": "",
        "navigationStart": 0,
        "unloadEventStart": -1,
        "unloadEventEnd": -1,
        "redirectStart": -1,
        "redirectEnd": -1,
        "fetchStart": -1,
        "domainLookupStart": -1,
        "domainLookupEnd": -1,
        "connectStart": -1,
        "connectEnd": -1,
        "secureConnectionStart": -1,
        "requestStart": -1,
        "responseStart": -1,
        "responseEnd": -1,
        "domLoading": -1,
        "domInteractive": -1,
        "domContentLoadedEventStart": -1,
        "domContentLoadedEventEnd": -1,
        "domComplete": -1,
        "loadEventStart": -1,
        "loadEventEnd": -1
    }]
    result = runner.parse_log(mock_log, mock_testcase)
    assert (expected == list(result))


def test_manifest_loader():

    text = '''
http://localhost/page_load_test/tp5n/163.com/www.163.com/index.html
http://localhost/page_load_test/tp5n/56.com/www.56.com/index.html

http://localhost/page_load_test/tp5n/aljazeera.net/aljazeera.net/portal.html
# Disabled! http://localhost/page_load_test/tp5n/aljazeera.net/aljazeera.net/portal.html
'''
    expected = [
        ("http://localhost/page_load_test/tp5n/163.com/www.163.com/index.html", False),
        ("http://localhost/page_load_test/tp5n/56.com/www.56.com/index.html", False),
        ("http://localhost/page_load_test/tp5n/aljazeera.net/aljazeera.net/portal.html", False)
    ]
    assert (expected == list(runner.parse_manifest(text)))


def test_manifest_loader_async():

    text = '''
http://localhost/page_load_test/tp5n/163.com/www.163.com/index.html
async http://localhost/page_load_test/tp5n/56.com/www.56.com/index.html
'''
    expected = [
        ("http://localhost/page_load_test/tp5n/163.com/www.163.com/index.html", False),
        ("http://localhost/page_load_test/tp5n/56.com/www.56.com/index.html", True),
    ]
    assert (expected == list(runner.parse_manifest(text)))


def test_filter_result_by_manifest():
    input_json = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/content.html",
        "domComplete": 1460358389000,
    }, {
        "testcase": "non-existing-html",
        "domComplete": 1460358389000,
    }, {
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": 1460358389000,
    }]

    expected = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": 1460358389000,
    }]

    manifest = [
        ("http://localhost:8000/page_load_test/56.com/www.56.com/index.html", False)
    ]

    assert (expected == runner.filter_result_by_manifest(input_json, manifest))


def test_filter_result_by_manifest_error():
    input_json = [{
        "testcase": "1.html",
        "domComplete": 1460358389000,
    }]

    manifest = [
        ("1.html", False),
        ("2.html", False)
    ]

    with pytest.raises(Exception) as execinfo:
        runner.filter_result_by_manifest(input_json, manifest)
    assert "Missing test result" in str(execinfo.value)


def test_take_result_median_odd():
    input_json = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": 1460358389001,
        "domLoading": 1460358380002
    }, {
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": 1460358389002,
        "domLoading": 1460358380001
    }, {
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": 1460358389003,
        "domLoading": 1460358380003
    }]

    expected = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": 1460358389002,
        "domLoading": 1460358380002
    }]

    assert (expected == runner.take_result_median(input_json, len(input_json)))


def test_take_result_median_even():
    input_json = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": 1460358389001,
        "domLoading": 1460358380002
    }, {
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": 1460358389002,
        "domLoading": 1460358380001
    }]

    expected = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": 1460358389001.5,
        "domLoading": 1460358380001.5
    }]

    assert (expected == runner.take_result_median(input_json, len(input_json)))


def test_take_result_median_error():
    input_json = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": None,
        "domLoading": 1460358380002
    }, {
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": 1460358389002,
        "domLoading": 1460358380001
    }]

    expected = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": 1460358389002,
        "domLoading": 1460358380001.5
    }]

    assert (expected == runner.take_result_median(input_json, len(input_json)))


def test_log_result():
    results = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": -1
    }, {
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "domComplete": -1
    }, {
        "testcase": "http://localhost:8000/page_load_test/104.com/www.104.com/index.html",
        "domComplete": 123456789
    }]

    expected = """
========================================
Total 3 tests; 1 succeeded, 2 failed.

Failure summary:
 - http://localhost:8000/page_load_test/56.com/www.56.com/index.html
========================================
"""
    assert (expected == runner.format_result_summary(results))
