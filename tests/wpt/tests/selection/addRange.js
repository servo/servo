"use strict";

function testAddRange(exception, range, endpoints, qualifier, testName) {
    if (!isSelectableNode(endpoints[0]) || !isSelectableNode(endpoints[2])) {
        testAddRangeDoesNothing(exception, range, endpoints, qualifier, testName);
        return;
    }

    test(function() {
        assert_equals(exception, null, "Test setup must not throw exceptions");

        selection.addRange(range);

        assert_equals(range.startContainer, endpoints[0],
            "addRange() must not modify the startContainer of the Range it's given");
        assert_equals(range.startOffset, endpoints[1],
            "addRange() must not modify the startOffset of the Range it's given");
        assert_equals(range.endContainer, endpoints[2],
            "addRange() must not modify the endContainer of the Range it's given");
        assert_equals(range.endOffset, endpoints[3],
            "addRange() must not modify the endOffset of the Range it's given");
    }, testName + ": " + qualifier + " addRange() must not throw exceptions or modify the range it's given");

    test(function() {
        assert_equals(exception, null, "Test setup must not throw exceptions");

        assert_equals(selection.rangeCount, 1, "rangeCount must be 1");
    }, testName + ": " + qualifier + " addRange() must result in rangeCount being 1");

    // From here on out we check selection.getRangeAt(selection.rangeCount - 1)
    // so as not to double-fail Gecko.

    test(function() {
        assert_equals(exception, null, "Test setup must not throw exceptions");
        assert_not_equals(selection.rangeCount, 0, "Cannot proceed with tests if rangeCount is 0");

        var newRange = selection.getRangeAt(selection.rangeCount - 1);

        assert_not_equals(newRange, null,
            "getRangeAt(rangeCount - 1) must not return null");
        assert_equals(typeof newRange, "object",
            "getRangeAt(rangeCount - 1) must return an object");

        assert_equals(newRange.startContainer, range.startContainer,
            "startContainer of the Selection's last Range must match the added Range");
        assert_equals(newRange.startOffset, range.startOffset,
            "startOffset of the Selection's last Range must match the added Range");
        assert_equals(newRange.endContainer, range.endContainer,
            "endContainer of the Selection's last Range must match the added Range");
        assert_equals(newRange.endOffset, range.endOffset,
            "endOffset of the Selection's last Range must match the added Range");
    }, testName + ": " + qualifier + " addRange() must result in the selection's last range having the specified endpoints");

    test(function() {
        assert_equals(exception, null, "Test setup must not throw exceptions");
        assert_not_equals(selection.rangeCount, 0, "Cannot proceed with tests if rangeCount is 0");

        assert_equals(selection.getRangeAt(selection.rangeCount - 1), range,
            "getRangeAt(rangeCount - 1) must return the same object we added");
    }, testName + ": " + qualifier + " addRange() must result in the selection's last range being the same object we added");

    // Let's not test many different modifications -- one should be enough.
    test(function() {
        assert_equals(exception, null, "Test setup must not throw exceptions");
        assert_not_equals(selection.rangeCount, 0, "Cannot proceed with tests if rangeCount is 0");

        if (range.startContainer == paras[0].firstChild
        && range.startOffset == 0
        && range.endContainer == paras[0].firstChild
        && range.endOffset == 2) {
            // Just in case . . .
            range.setStart(paras[0].firstChild, 1);
        } else {
            range.setStart(paras[0].firstChild, 0);
            range.setEnd(paras[0].firstChild, 2);
        }

        var newRange = selection.getRangeAt(selection.rangeCount - 1);

        assert_equals(newRange.startContainer, range.startContainer,
            "After mutating the " + qualifier + " added Range, startContainer of the Selection's last Range must match the added Range");
        assert_equals(newRange.startOffset, range.startOffset,
            "After mutating the " + qualifier + " added Range, startOffset of the Selection's last Range must match the added Range");
        assert_equals(newRange.endContainer, range.endContainer,
            "After mutating the " + qualifier + " added Range, endContainer of the Selection's last Range must match the added Range");
        assert_equals(newRange.endOffset, range.endOffset,
            "After mutating the " + qualifier + " added Range, endOffset of the Selection's last Range must match the added Range");
    }, testName + ": modifying the " + qualifier + " added range must modify the Selection's last Range");

    // Now test the other way too.
    test(function() {
        assert_equals(exception, null, "Test setup must not throw exceptions");
        assert_not_equals(selection.rangeCount, 0, "Cannot proceed with tests if rangeCount is 0");

        var newRange = selection.getRangeAt(selection.rangeCount - 1);

        if (newRange.startContainer == paras[0].firstChild
        && newRange.startOffset == 4
        && newRange.endContainer == paras[0].firstChild
        && newRange.endOffset == 6) {
            newRange.setStart(paras[0].firstChild, 5);
        } else {
            newRange.setStart(paras[0].firstChild, 4);
            newRange.setStart(paras[0].firstChild, 6);
        }

        assert_equals(newRange.startContainer, range.startContainer,
            "After " + qualifier + " addRange(), after mutating the Selection's last Range, startContainer of the Selection's last Range must match the added Range");
        assert_equals(newRange.startOffset, range.startOffset,
            "After " + qualifier + " addRange(), after mutating the Selection's last Range, startOffset of the Selection's last Range must match the added Range");
        assert_equals(newRange.endContainer, range.endContainer,
            "After " + qualifier + " addRange(), after mutating the Selection's last Range, endContainer of the Selection's last Range must match the added Range");
        assert_equals(newRange.endOffset, range.endOffset,
            "After " + qualifier + " addRange(), after mutating the Selection's last Range, endOffset of the Selection's last Range must match the added Range");
    }, testName + ": modifying the Selection's last Range must modify the " + qualifier + " added Range");
}

function testAddRangeDoesNothing(exception, range, endpoints, qualifier, testName) {
    test(function() {
        assert_equals(exception, null, "Test setup must not throw exceptions");

        assertSelectionNoChange(function() { selection.addRange(range); });
        assert_equals(range.startContainer, endpoints[0],
            "addRange() must not modify the startContainer of the Range it's given");
        assert_equals(range.startOffset, endpoints[1],
            "addRange() must not modify the startOffset of the Range it's given");
        assert_equals(range.endContainer, endpoints[2],
            "addRange() must not modify the endContainer of the Range it's given");
        assert_equals(range.endOffset, endpoints[3],
            "addRange() must not modify the endOffset of the Range it's given");
    }, testName + ": " + qualifier + " addRange() must do nothing");
}

// Do only n evals, not n^2
var testRangesEvaled = testRanges.map(eval);

// Run a subset of all of addRange tests.
// Huge number of tests in a single file causes problems. Each of
// addRange-NN.html runs a part of them.
//
// startIndex - Start index in testRanges array
// optionalEndIndex - End index in testRanges array + 1. If this argument is
//     omitted, testRanges.length is applied.
function testAddRangeSubSet(startIndex, optionalEndIndex) {
    var endIndex = optionalEndIndex === undefined ? testRanges.length : optionalEndIndex;
    if (startIndex < 0 || startIndex >= testRanges.length)
        throw "Sanity check: Specified index is invalid.";
    if (endIndex < 0 || endIndex > testRanges.length)
        throw "Sanity check: Specified index is invalid.";

    for (var i = startIndex; i < endIndex; i++) {
        for (var j = 0; j < testRanges.length; j++) {
            var testName = "Range " + i + " " + testRanges[i]
                + " followed by Range " + j + " " + testRanges[j];

            var exception = null;
            try {
                selection.removeAllRanges();

                var endpoints1 = testRangesEvaled[i];
                var range1 = ownerDocument(endpoints1[0]).createRange();
                range1.setStart(endpoints1[0], endpoints1[1]);
                range1.setEnd(endpoints1[2], endpoints1[3]);

                if (range1.startContainer !== endpoints1[0]) {
                    throw "Sanity check: the first Range we created must have the desired startContainer";
                }
                if (range1.startOffset !== endpoints1[1]) {
                    throw "Sanity check: the first Range we created must have the desired startOffset";
                }
                if (range1.endContainer !== endpoints1[2]) {
                    throw "Sanity check: the first Range we created must have the desired endContainer";
                }
                if (range1.endOffset !== endpoints1[3]) {
                    throw "Sanity check: the first Range we created must have the desired endOffset";
                }

                var endpoints2 = testRangesEvaled[j];
                var range2 = ownerDocument(endpoints2[0]).createRange();
                range2.setStart(endpoints2[0], endpoints2[1]);
                range2.setEnd(endpoints2[2], endpoints2[3]);

                if (range2.startContainer !== endpoints2[0]) {
                    throw "Sanity check: the second Range we created must have the desired startContainer";
                }
                if (range2.startOffset !== endpoints2[1]) {
                    throw "Sanity check: the second Range we created must have the desired startOffset";
                }
                if (range2.endContainer !== endpoints2[2]) {
                    throw "Sanity check: the second Range we created must have the desired endContainer";
                }
                if (range2.endOffset !== endpoints2[3]) {
                    throw "Sanity check: the second Range we created must have the desired endOffset";
                }
            } catch (e) {
                exception = e;
            }

            testAddRange(exception, range1, endpoints1, "first", testName);
            if (selection.rangeCount > 0)
                testAddRangeDoesNothing(exception, range2, endpoints2, "second", testName);
        }
    }
}

