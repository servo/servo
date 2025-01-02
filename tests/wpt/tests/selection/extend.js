"use strict";

// Also test a selection with no ranges
testRanges.unshift("[]");

// Run a subset of all of extend tests.
// Huge number of tests in a single file causes problems. Each of
// extend-NN.html runs a part of them.
//
// startIndex - Start index in testRanges array
// optionalEndIndex - End index in testRanges array + 1. If this argument is
//     omitted, testRanges.length is applied.
function testExtendSubSet(startIndex, optionalEndIndex) {
    var endIndex = optionalEndIndex === undefined ? testRanges.length : optionalEndIndex;
    if (startIndex < 0 || startIndex >= testRanges.length)
        throw "Sanity check: Specified index is invalid.";
    if (endIndex < 0 || endIndex > testRanges.length)
        throw "Sanity check: Specified index is invalid.";

    // We test Selections that go both forwards and backwards here.  In the
    // latter case we need to use extend() to force it to go backwards, which is
    // fair enough, since that's what we're testing.  We test collapsed
    // selections only once.
    for (var i = startIndex; i < endIndex; i++) {
        var endpoints = eval(testRanges[i]);
        // We can't test extend() with unselectable endpoints.
        if (!isSelectableNode(endpoints[0]) || !isSelectableNode(endpoints[2]))
            continue;
        for (var j = 0; j < testPoints.length; j++) {
            if (endpoints[0] == endpoints[2]
            && endpoints[1] == endpoints[3]) {
                // Test collapsed selections only once
                test(function() {
                    setSelectionForwards(endpoints);
                    testExtend(endpoints, eval(testPoints[j]));
                }, "extend() with range " + i + " " + testRanges[i]
                + " and point " + j + " " + testPoints[j]);
            } else {
                test(function() {
                    setSelectionForwards(endpoints);
                    testExtend(endpoints, eval(testPoints[j]));
                }, "extend() forwards with range " + i + " " + testRanges[i]
                + " and point " + j + " " + testPoints[j]);

                test(function() {
                    setSelectionBackwards(endpoints);
                    testExtend(endpoints, eval(testPoints[j]));
                }, "extend() backwards with range " + i + " " + testRanges[i]
                + " and point " + j + " " + testPoints[j]);
            }
        }
    }
}

function testExtend(endpoints, target) {
    assert_equals(getSelection().rangeCount, endpoints.length/4,
        "Sanity check: rangeCount must be correct");

    var node = target[0];
    var offset = target[1];

    // "If node's root is not the document associated with the context object,
    // abort these steps."
    if (!document.contains(node)) {
        assertSelectionNoChange(function() {
            selection.extend(node, offset);
        });
        return;
    }

    // "If the context object's range is null, throw an InvalidStateError
    // exception and abort these steps."
    if (getSelection().rangeCount == 0) {
        assert_throws_dom("INVALID_STATE_ERR", function() {
            selection.extend(node, offset);
        }, "extend() when rangeCount is 0 must throw InvalidStateError");
        return;
    }

    assert_equals(getSelection().getRangeAt(0).startContainer, endpoints[0],
        "Sanity check: startContainer must be correct");
    assert_equals(getSelection().getRangeAt(0).startOffset, endpoints[1],
        "Sanity check: startOffset must be correct");
    assert_equals(getSelection().getRangeAt(0).endContainer, endpoints[2],
        "Sanity check: endContainer must be correct");
    assert_equals(getSelection().getRangeAt(0).endOffset, endpoints[3],
        "Sanity check: endOffset must be correct");

    // "Let anchor and focus be the context object's anchor and focus, and let
    // new focus be the boundary point (node, offset)."
    var anchorNode = getSelection().anchorNode;
    var anchorOffset = getSelection().anchorOffset;
    var focusNode = getSelection().focusNode;
    var focusOffset = getSelection().focusOffset;

    // "Let new range be a new range."
    //
    // We'll always be setting either new range's start or its end to new
    // focus, so we'll always throw at some point.  Test that now.
    //
    // From DOM4's "set the start or end of a range": "If node is a doctype,
    // throw an "InvalidNodeTypeError" exception and terminate these steps."
    if (node.nodeType == Node.DOCUMENT_TYPE_NODE) {
        assert_throws_dom("INVALID_NODE_TYPE_ERR", function() {
            selection.extend(node, offset);
        }, "extend() to a doctype must throw InvalidNodeTypeError");
        return;
    }

    // From DOM4's "set the start or end of a range": "If offset is greater
    // than node's length, throw an "IndexSizeError" exception and terminate
    // these steps."
    //
    // FIXME: We should be casting offset to an unsigned int per WebIDL.  Until
    // we do, we need the offset < 0 check too.
    if (offset < 0 || offset > getNodeLength(node)) {
        assert_throws_dom("INDEX_SIZE_ERR", function() {
            selection.extend(node, offset);
        }, "extend() to an offset that's greater than node length (" + getNodeLength(node) + ") must throw IndexSizeError");
        return;
    }

    // Now back to the editing spec.
    var originalRange = getSelection().getRangeAt(0);

    // "If node's root is not the same as the context object's range's root,
    // set new range's start and end to (node, offset)."
    //
    // "Otherwise, if anchor is before or equal to new focus, set new range's
    // start to anchor, then set its end to new focus."
    //
    // "Otherwise, set new range's start to new focus, then set its end to
    // anchor."
    //
    // "Set the context object's range to new range."
    //
    // "If new focus is before anchor, set the context object's direction to
    // backwards. Otherwise, set it to forwards."
    //
    // The upshot of all these is summed up by just testing the anchor and
    // offset.
    getSelection().extend(node, offset);

    if (furthestAncestor(anchorNode) == furthestAncestor(node)) {
        assert_equals(getSelection().anchorNode, anchorNode,
            "anchorNode must not change if the node passed to extend() has the same root as the original range");
        assert_equals(getSelection().anchorOffset, anchorOffset,
            "anchorOffset must not change if the node passed to extend() has the same root as the original range");
    } else {
        assert_equals(getSelection().anchorNode, node,
            "anchorNode must be the node passed to extend() if it has a different root from the original range");
        assert_equals(getSelection().anchorOffset, offset,
            "anchorOffset must be the offset passed to extend() if the node has a different root from the original range");
    }
    assert_equals(getSelection().focusNode, node,
        "focusNode must be the node passed to extend()");
    assert_equals(getSelection().focusOffset, offset,
        "focusOffset must be the offset passed to extend()");
    assert_not_equals(getSelection().getRangeAt(0), originalRange,
        "extend() must replace any existing range with a new one, not mutate the existing one");
}
