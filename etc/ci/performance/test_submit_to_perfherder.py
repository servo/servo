import submit_to_perfherder


def test_format_testcase_name():
    assert('about:blank' == submit_to_perfherder.format_testcase_name('about:blank'))
    assert('163.com' == submit_to_perfherder.format_testcase_name('http://localhost:8000/page_load_test/163.com/p.mail.163.com/mailinfo/shownewmsg_www_1222.htm.html'))
    assert('12345678902234567890323456789042345678905234567890623456789072345678908234567890' == submit_to_perfherder.format_testcase_name('123456789022345678903234567890423456789052345678906234567890723456789082345678909234567890'))


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
            "testcase": "http://localhost:8000/page_load_test/163.com/p.mail.163.com/mailinfo/shownewmsg_www_1222.htm.html",
            "domComplete": 1460444948000
        }
    ]

    expected = {
        "performance_data": {
            # TODO: can we create a framwork on treeherder
            # that is not `talos`?
            "framework": {"name": "talos"},
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
    '''
    expected = {
        "performance_data": {
            # TODO: can we create a framwork on treeherder
            # that is not `talos`?
            "framework": {"name": "talos"},
            "suites": [
                {
                    "name": "about:blank",
                    "value": 1460444931000,
                    "subtests": [
                        {"name":"unloadEventStart", "value": None},
                        {"name":"domLoading", "value": 1460444930000},
                        {"name":"fetchStart", "value": None},
                        {"name":"responseStart", "value": None},
                        {"name":"loadEventEnd", "value": None},
                        {"name":"connectStart", "value": None},
                        {"name":"domainLookupStart", "value": None},
                        {"name":"redirectStart", "value": None},
                        {"name":"domContentLoadedEventEnd", "value": 1460444930000},
                        {"name":"requestStart", "value": None},
                        {"name":"secureConnectionStart", "value": None},
                        {"name":"connectEnd", "value": None},
                        {"name":"navigationStart", "value": 1460444930},
                        {"name":"loadEventStart", "value": None},
                        {"name":"domInteractive", "value": 1460444930000},
                        {"name":"domContentLoadedEventStart", "value": 1460444930000},
                        {"name":"redirectEnd", "value": None},
                        {"name":"domainLookupEnd", "value": None},
                        {"name":"unloadEventEnd", "value": None},
                        {"name":"responseEnd", "value": None},
                        {"name":"domComplete", "value": 1460444931000},
                    ]
                },
                {
                    "name": "http://localhost:8000/page_load_test/163.com/p.mail.163.com/mailinfo/shownewmsg_www_1222.htm.html",
                    "value": 1460444948000,
                    "subtests": [
                        {"name":"unloadEventStart", "value": None},
                        {"name":"domLoading", "value": 1460444934000},
                        {"name":"fetchStart", "value": None},
                        {"name":"responseStart", "value": None},
                        {"name":"loadEventEnd", "value": None},
                        {"name":"connectStart", "value": None},
                        {"name":"domainLookupStart", "value": None},
                        {"name":"redirectStart", "value": None},
                        {"name":"domContentLoadedEventEnd", "value": 1460444946000},
                        {"name":"requestStart", "value": None},
                        {"name":"secureConnectionStart", "value": None},
                        {"name":"connectEnd", "value": None},
                        {"name":"navigationStart", "value": 1460444934},
                        {"name":"loadEventStart", "value": None},
                        {"name":"domInteractive", "value": 1460444946000},
                        {"name":"domContentLoadedEventStart", "value": 1460444946000},
                        {"name":"redirectEnd", "value": None},
                        {"name":"domainLookupEnd", "value": None},
                        {"name":"unloadEventEnd", "value": None},
                        {"name":"responseEnd", "value": None},
                        {"name":"domComplete", "value": 1460444948000 },
                    ]
                }
            ]
        }
    }
    '''
    result = submit_to_perfherder.format_perf_data(mock_result)
    assert(expected == result)


def test_format_bad_perf_data():
    mock_result = [
        {
            "navigationStart": 1460444930000,
            "testcase": "about:blank",
            "domComplete": 0
        },
        {
            "navigationStart": 1460444934000,
            "testcase": "http://localhost:8000/page_load_test/163.com/p.mail.163.com/mailinfo/shownewmsg_www_1222.htm.html",
            "domComplete": 1460444948000
        }
    ]

    expected = {
        "performance_data": {
            # TODO: can we create a framwork on treeherder
            # that is not `talos`?
            "framework": {"name": "talos"},
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
    assert(expected == result)
