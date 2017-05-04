#!/usr/bin/env python
# coding=utf-8
#
#  Copyright © 2013 Hewlett-Packard Development Company, L.P.
#
#  This work is distributed under the W3C® Software License [1]
#  in the hope that it will be useful, but WITHOUT ANY
#  WARRANTY; without even the implied warranty of
#  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
#
#  [1] http://www.w3.org/Consortium/Legal/2002/copyright-software-20021231
#

import sys
import os
import glob
import json
import exceptions
import collections

from apiclient import uritemplate
from apiclient import apiclient


def runTests(testFileSearch):
    for testFilePath in glob.glob(testFileSearch):
        print 'Running tests from: ' + testFilePath
        with open(testFilePath) as testFile:
            testData = json.load(testFile, object_pairs_hook = collections.OrderedDict)
            for testSetName in testData:
                print testSetName + ':'
                testSet = testData[testSetName]
                vars = testSet['variables']
                for test in testSet['testcases']:
                    expectedResult = test[1]
                    try:
                        template = uritemplate.URITemplate(test[0])
                    except Exception as e:
                        if (expectedResult):
                            print '* FAIL: "' + test[0] + '" got: None, expected "' + expectedResult + '"'
                        else:
                            print '  PASS: "' + test[0] + '" == None'
                        continue
                    
                    result = template.expand(**vars)
                    if (isinstance(expectedResult, basestring)):
                        if (expectedResult != result):
                            print '* FAIL: "' + test[0] + '" got: "' + unicode(result) + '", expected "' + expectedResult + '"'
                            continue
                    elif (isinstance(expectedResult, list)):
                        for possibleResult in expectedResult:
                            if (possibleResult == result):
                                break
                        else:
                            print '* FAIL: "' + test[0] + '" got: "' + unicode(result) + '", expected:'
                            print " or\n".join(['    "' + possibleResult + '"' for possibleResult in expectedResult])
                            continue
                    elif (not expectedResult):
                        if (result):
                            print '* FAIL "' + test[0] + '" got: "' + unicode(result) + '", expected None'
                            continue
                    else:
                        print '** Unknown expected result type: ' + repr(expectedResult)
                    print '  PASS: "' + test[0] + '" == "' + result + '"'

def debugHook(type, value, tb):
    if hasattr(sys, 'ps1') or not sys.stderr.isatty():
        # we are in interactive mode or we don't have a tty-like
        # device, so we call the default hook
        sys.__excepthook__(type, value, tb)
    else:
        import traceback, pdb
        # we are NOT in interactive mode, print the exception...
        traceback.print_exception(type, value, tb)
        print
        # ...then start the debugger in post-mortem mode.
        pdb.pm()


if __name__ == "__main__":      # called from the command line
    sys.excepthook = debugHook
    
#    runTests(os.path.join('test', '*.json'))

#    runTests(os.path.join('uritemplate-test', 'spec-examples.json'))
#    runTests(os.path.join('uritemplate-test', '*.json'))
### more tests @ https://github.com/uri-templates/uritemplate-test


    github = apiclient.APIClient('https://api.github.com/', version = 'vnd.github.beta')
    print github.get('user_url', user = 'plinss').data

#    shepherd = apiclient.APIClient('https://api.csswg.org/shepherd/', version = 'vnd.csswg.shepherd.v1')
    shepherd = apiclient.APIClient('https://test.linss.com/shepherd/api', version = 'vnd.csswg.shepherd.v1')
    print shepherd.resourceNames
    specs = shepherd.resource('specifications')
    print specs.variables
#    print specs.hints.docs
    print shepherd.get('specifications', spec = 'compositing-1', anchors = False).data

    suites = shepherd.resource('test_suites')
    print suites.variables
    print shepherd.get('test_suites', spec = 'css-shapes-1').data


