"use strict";

function testCollapse(range, point, method) {
    selection.removeAllRanges();
    var addedRange;
    if (range) {
        addedRange = range.cloneRange();
        selection.addRange(addedRange);
    }

    if (point[0].nodeType == Node.DOCUMENT_TYPE_NODE) {
        assert_throws_dom("INVALID_NODE_TYPE_ERR", function() {
            selection[method](point[0], point[1]);
        }, "Must throw INVALID_NODE_TYPE_ERR when " + method + "()ing if the node is a DocumentType");
        return;
    }

    if (point[1] < 0 || point[1] > getNodeLength(point[0])) {
        assert_throws_dom("INDEX_SIZE_ERR", function() {
            selection[method](point[0], point[1]);
        }, "Must throw INDEX_SIZE_ERR when " + method + "()ing if the offset is negative or greater than the node's length");
        return;
    }

    if (!document.contains(point[0])) {
        assertSelectionNoChange(function() {
            selection[method](point[0], point[1]);
        });
        return;
    }

    selection[method](point[0], point[1]);

    assert_equals(selection.rangeCount, 1,
        "selection.rangeCount must equal 1 after " + method + "()");
    assert_equals(selection.focusNode, point[0],
        "focusNode must equal the node we " + method + "()d to");
    assert_equals(selection.focusOffset, point[1],
        "focusOffset must equal the offset we " + method + "()d to");
    assert_equals(selection.focusNode, selection.anchorNode,
        "focusNode and anchorNode must be equal after " + method + "()");
    assert_equals(selection.focusOffset, selection.anchorOffset,
        "focusOffset and anchorOffset must be equal after " + method + "()");
    if (range) {
        assert_equals(addedRange.startContainer, range.startContainer,
            method + "() must not change the startContainer of a preexisting Range");
        assert_equals(addedRange.endContainer, range.endContainer,
            method + "() must not change the endContainer of a preexisting Range");
        assert_equals(addedRange.startOffset, range.startOffset,
            method + "() must not change the startOffset of a preexisting Range");
        assert_equals(addedRange.endOffset, range.endOffset,
            method + "() must not change the endOffset of a preexisting Range");
    }
}

// Also test a selection with no ranges
testRanges.unshift("[]");

// Don't want to eval() each point a bazillion times
var testPointsCached = [];
for (var i = 0; i < testPoints.length; i++) {
    testPointsCached.push(eval(testPoints[i]));
}

// Run a subset of all of collapse tests.
// Huge number of tests in a single file causes problems. Each of
// collapse-NN.html runs a part of them.
//
// startIndex - Start index in testRanges array
// optionalEndIndex - End index in testRanges array + 1. If this argument is
//     omitted, testRanges.length is applied.
function testCollapseSubSet(startIndex, optionalEndIndex) {
    var endIndex = optionalEndIndex === undefined ? testRanges.length : optionalEndIndex;
    if (startIndex < 0 || startIndex >= testRanges.length)
        throw "Sanity check: Specified index is invalid.";
    if (endIndex < 0 || endIndex > testRanges.length)
        throw "Sanity check: Specified index is invalid.";

    var tests = [];
    for (var i = startIndex; i < endIndex; i++) {
        var endpoints = eval(testRanges[i]);
        var range;
        test(function() {
            if (endpoints.length) {
                range = ownerDocument(endpoints[0]).createRange();
                range.setStart(endpoints[0], endpoints[1]);
                range.setEnd(endpoints[2], endpoints[3]);
            } else {
                // Empty selection
                range = null;
            }
        }, "Set up range " + i + " " + testRanges[i]);
        for (var j = 0; j < testPoints.length; j++) {
            tests.push(["collapse() on " + testRanges[i] + " to " + testPoints[j],
                        range, testPointsCached[j], "collapse"]);
            tests.push(["setPosition() on " + testRanges[i] + " to " + testPoints[j],
                        range, testPointsCached[j], "setPosition"]);
        }
    }

    generate_tests(testCollapse, tests);
}

