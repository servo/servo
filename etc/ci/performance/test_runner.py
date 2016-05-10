import runner


def test_log_parser():
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
    result = runner.parse_log(mock_log)
    assert(expected == list(result))


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
    expected = [{
        "testcase": "http://localhost:8000/page_load_test/56.com/www.56.com/content.html",
        "navigationStart": 1460358300,
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
    }, {
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
    result = runner.parse_log(mock_log)
    assert(expected == list(result))


def test_manifest_loader():

    text = '''
http://localhost/page_load_test/tp5n/163.com/www.163.com/index.html
http://localhost/page_load_test/tp5n/56.com/www.56.com/index.html

http://localhost/page_load_test/tp5n/aljazeera.net/aljazeera.net/portal.html
# Disabled! http://localhost/page_load_test/tp5n/aljazeera.net/aljazeera.net/portal.html
'''
    expected = [
        "http://localhost/page_load_test/tp5n/163.com/www.163.com/index.html",
        "http://localhost/page_load_test/tp5n/56.com/www.56.com/index.html",
        "http://localhost/page_load_test/tp5n/aljazeera.net/aljazeera.net/portal.html"
    ]
    assert(expected == list(runner.parse_manifest(text)))


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
        "http://localhost:8000/page_load_test/56.com/www.56.com/index.html",
        "http://localhost:8000/page_load_test/5566.com/www.5566.com/index.html"
    ]

    assert(expected == runner.filter_result_by_manifest(input_json, manifest))


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

    assert(expected == runner.take_result_median(input_json, len(input_json)))


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

    assert(expected == runner.take_result_median(input_json, len(input_json)))


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

    assert(expected == runner.take_result_median(input_json, len(input_json)))
