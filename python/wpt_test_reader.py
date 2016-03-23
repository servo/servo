# Program that reads in WPT tests from stdout

import argparse
import sys
import json

class TestResult:

    def __init__(self):
        self.subtest = False
        self.result = ''
        self.expected_result = ''
        self.file = ''
        self.info = ''
        self.reason_for_fail = ''
        self.traceback = ''

def main(logfile):
    failed_tests = []
    timedout_tests = []
    notrun_tests = []
    crashed_tests = []
    passed_tests = []
    ok_tests = []

    results = []

    subtest = False
    subtest_path = ''
    subtest_traceback = ''

    triangle = u'\u25b6'
    arrow = u'\u2192'
    vertical_bar = u'\u2502'
    corner = u'\u2514'

    for line in sys.stdin:
    # check for triangle because new tests start with that, and then write test
    # data to appropriate array
        print(line)
        if triangle in line:
            path = ''
            subtest = False
            for i,  val in enumerate(results):
                test_result = {'subtest': results[i].subtest, 'result': results[i].result, 
                            'expected_result': results[i].expected_result,
                            'file': results[i].file, 'info': results[i].file,
                            'info': results[i].info, 'reason': results[i].reason_for_fail,
                            'traceback': results[i].traceback
                            }
                if results[i].result == 'FAIL':
                    failed_tests.append(test_result)
                elif results[i].result == 'TIMEOUT':
                    timedout_tests.append(test_result)
                elif results[i].result == 'NOTRUN':
                    notrun_tests.append(test_result)
                elif results[i].result == 'CRASH':
                    crashed_tests.append(test_result)
                elif results[i].result == 'PASS':
                    passed_tests.append(test_result)
                elif results[i].result == 'OK':
                    ok_tests.append(test_result)

            # clear results so new test data can be recorded 
            del results[:]
            elements = line.split()

            # figure out test type
            if elements[1] == 'Unexpected':
                subtest = True
                subtest_path = elements[len(elements) - 1].split(':')[0].lstrip()
            elif elements[1] == 'TIMEOUT':
                results.append(TestResult())
                results[-1].expected_result = elements[3].split(']')[0].lstrip()
                results[-1].file = elements[4]
                results[-1].result = 'TIMEOUT'
            elif elements[1] == 'OK':
                results.append(TestResult())
                results[-1].expected_result = elements[3].split(']')[0].lstrip()
                results[-1].file = elements[4]
                results[-1].result = 'OK'
            elif elements[1] == 'PASS':
                results.append(TestResult())
                results[-1].expected_result = elements[3].split(']')[0].lstrip()
                results[-1].file = elements[4]
                results[-1].result = 'PASS'
            elif elements[1] == 'CRASH':
                results.append(TestResult())
                results[-1].expected_result = elements[3].split(']')[0].lstrip()
                results[-1].file = elements[4]
                results[-1].result = 'CRASH'
            elif elements[1] == 'FAIL':
                results.append(TestResult())
                results[-1].expected_result = elements[3].split(']')[0].lstrip()
                results[-1].file = elements[4]
                results[-1].result = 'FAIL'
        else:
            # record test information
            elements = line.split()
            if subtest:
                if line != '\n' and len(elements) > 1:
                    if elements[1] == 'FAIL':
                        results.append(TestResult())
                        results[-1].subtest = subtest
                        results[-1].path = subtest_path
                        results[-1].result = 'FAIL'
                        results[-1].expected_result = elements[3].split(']')[1].lstrip()
                        results[-1].info = line.split(']')[1].lstrip()
                    elif elements[1] == 'NOTRUN':
                        results.append(TestResult())
                        results[-1].subtest = subtest
                        results[-1].path = subtest_path
                        results[-1].result = 'NOTRUN'
                        results[-1].expected_result = elements[3].split(']')[1].lstrip()
                        results[-1].info = line.split(']')[1].lstrip()
                    elif elements[1] == 'TIMEOUT':
                        results.append(TestResult())
                        results[-1].subtest = subtest
                        results[-1].path = subtest_path
                        results[-1].result = 'TIMEOUT'
                        results[-1].expected_result = elements[3].split(']')[1].lstrip()
                        results[-1].info = line.split(']')[1].lstrip()
                    elif elements[1] == arrow:
                        results[-1].reason_for_fail = line.split(arrow)[1].lstrip()
                    else:
                        for i, val in enumerate(results):
                            if elements[0] == vertical_bar:
                                results[i].traceback += line.split(vertical_bar)[1].lstrip()
                            elif elements[0] == corner:
                                results[i].traceback += line.split(corner)[1].lstrip()
                            else:
                                if len(results) > 0:
                                    results[i].traceback += line
            # information for tests that aren't subtests
            else:
                elements = line.split()
                if line != '\n' and len(elements) > 1:
                    if elements[1] == arrow:
                        results[-1].reason_for_fail = line.split(arrow)[1].lstrip()
                    else:
                        if elements[0] == vertical_bar:
                            results[-1].traceback += line.split(vertical_bar)[1].lstrip()
                        elif elements[0] == corner:
                            results[-1].traceback += line.split(corner)[1].lstrip()
                        else:
                            if len(results) > 0:
                                results[-1].traceback += line
    
    data = {'FAIL': failed_tests, 'CRASH': crashed_tests, 'PASS': passed_tests,
            'TIMEOUT': timedout_tests, 'NOTRUN': notrun_tests, 'OK': ok_tests}

    with open(logfile, 'w') as outfile:
        json.dump(data, outfile)

        
if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument('--logfile', type=str, help='Output file for logs',
            default='wpt_test_results.json')
    args = parser.parse_args()
    logfile = args.logfile
    main(logfile)
