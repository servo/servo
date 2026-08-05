// OHOS WebDriver Test Result Parser
// Parses WPT (Web Platform Test) test results from the DOM and extracts info
// Executed in the browser context via WebDriver to analyze
// the test results displayed on the page after WPT tests complete.

try {
    var result = {
        title: document.title,
        readyState: document.readyState,
        bodyText: document.body ? document.body.textContent : ''
    };

    var bodyText = result.bodyText || '';
    var titleText = result.title || '';

    if (bodyText.includes('Harness status: OK')) {
        // Look for test result patterns like "X Pass Y Fail"
        var passMatch = bodyText.match(/(\d+)\s+Pass/i);
        var failMatch = bodyText.match(/(\d+)\s+Fail/i);

        var passCount = passMatch ? parseInt(passMatch[1]) : 0;
        var failCount = failMatch ? parseInt(failMatch[1]) : 0;

        result.passCount = passCount;
        result.failCount = failCount;
        result.failingTests = [];

        // Parse individual test results by splitting by "Fail" keyword
        var testSections = bodyText.split('Fail');

        for (var i = 1; i < testSections.length; i++) {
            var section = testSections[i];
            if (!section || section.trim().length === 0) continue;

            // Find the end of this test section (next "Pass" or "Fail" or "Asserts run")
            var endMarkers = ['Pass', 'Asserts run'];
            var endIndex = section.length;

            for (var j = 0; j < endMarkers.length; j++) {
                var markerIndex = section.indexOf(endMarkers[j]);
                if (markerIndex !== -1 && markerIndex < endIndex) {
                    endIndex = markerIndex;
                }
            }

            var testContent = section.substring(0, endIndex).trim();
            if (!testContent) continue;

            // Error message patterns to split test name from error
            var errorPatterns = [
                'promise_test:',
                'assert_equals:',
                'assert_not_equals:',
                'assert_less_than:',
                'assert_greater_than:',
                'assert_true:',
                'assert_false:',
                'TypeError:',
                'ReferenceError:',
                '@http://'  // Add pattern for location references
            ];

            var testName = '';
            var errorMessage = '';
            var splitIndex = -1;

            for (var k = 0; k < errorPatterns.length; k++) {
                var patternIndex = testContent.indexOf(errorPatterns[k]);
                if (patternIndex !== -1) {
                    if (splitIndex === -1 || patternIndex < splitIndex) {
                        splitIndex = patternIndex;
                    }
                }
            }

            if (splitIndex !== -1) {
                testName = testContent.substring(0, splitIndex).trim();
                errorMessage = testContent.substring(splitIndex).trim();
            } else {
                // No clear error pattern, use first line as test name and rest as error
                var lines = testContent.split('\n');
                testName = lines[0] ? lines[0].trim() : '';
                errorMessage = lines.slice(1).join(' ').trim();
                
                // If still no split, try splitting on common delimiters
                if (!errorMessage && testName) {
                    // Try to find the end of the test name by looking for specific patterns
                    var delimiterPatterns = [
                        ' got disallowed value',
                        ' expected ',
                        ' assert_',
                        ' TypeError',
                        ' ReferenceError'
                    ];
                    
                    for (var d = 0; d < delimiterPatterns.length; d++) {
                        var delimIndex = testName.indexOf(delimiterPatterns[d]);
                        if (delimIndex !== -1) {
                            errorMessage = testName.substring(delimIndex).trim();
                            testName = testName.substring(0, delimIndex).trim();
                            break;
                        }
                    }
                }
            }

            // Clean up test name
            if (!testName || testName.length === 0) {
                testName = 'Unnamed Test #' + result.failingTests.length;
            }

            var isAssertionLine = false;
            var isFilePathLine = false;

            // Check if it's an assertion line (starts with assert_ and has parentheses and file reference)
            if (testName.indexOf('assert_') === 0 && 
                testName.indexOf('(') !== -1 && 
                testName.indexOf(')') !== -1 &&
                testName.indexOf('.html:') !== -1) {
                isAssertionLine = true;
            }

            // Check if it's a file path line (starts with /css/ or has only file reference)
            if (testName.indexOf('/css/') === 0 ||
                (testName.indexOf('.html:') !== -1 && testName.length < 60 && testName.indexOf(' ') === -1)) {
                isFilePathLine = true;
            }

            // Additional check: if it looks like just an assertion call with file location
            if (testName.indexOf('assert_') === 0 && 
                testName.indexOf('(') !== -1 && 
                testName.indexOf(')') !== -1 && 
                testName.indexOf(',') !== -1) {
                isAssertionLine = true;
            }

            if (errorMessage.length > 250) {
                errorMessage = errorMessage.substring(0, 250) + '...';
            }

            // Only add if we have meaningful content, avoid assertion lines, and prevent duplicates
            if (testName && errorMessage && !isAssertionLine && !isFilePathLine) {
                var isDuplicate = false;
                for (var m = 0; m < result.failingTests.length; m++) {
                    if (result.failingTests[m].name === testName) {
                        isDuplicate = true;
                        break;
                    }
                }

                if (!isDuplicate) {
                    result.failingTests.push({
                        name: testName,
                        error: errorMessage
                    });
                }
            }
        }

        if (failCount > 0) {
            result.status = 'FAIL';
        } else if (passCount > 0) {
            result.status = 'PASS';
        } else {
            result.status = 'UNKNOWN';
        }
    } else if (bodyText.includes('PASS') || titleText.includes('PASS')) {
        result.status = 'PASS';
    } else if (bodyText.includes('FAIL') || titleText.includes('FAIL')) {
        result.status = 'FAIL';
    } else {
        result.status = 'UNKNOWN';
    }

    return result;
} catch (e) {
    return {status: 'ERROR', title: document.title, error: e.message};
}
