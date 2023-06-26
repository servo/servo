"use strict";
// TODO: iframes, contenteditable/designMode

// Everything is done in functions in this test harness, so we have to declare
// all the variables before use to make sure they can be reused.
var selection;
var testDiv, paras, detachedDiv, detachedPara1, detachedPara2,
    foreignDoc, foreignPara1, foreignPara2, xmlDoc, xmlElement,
    detachedXmlElement, detachedTextNode, foreignTextNode,
    detachedForeignTextNode, xmlTextNode, detachedXmlTextNode,
    processingInstruction, detachedProcessingInstruction, comment,
    detachedComment, foreignComment, detachedForeignComment, xmlComment,
    detachedXmlComment, docfrag, foreignDocfrag, xmlDocfrag, doctype,
    foreignDoctype, xmlDoctype;
var testRanges, testPoints, testNodes;

function setupRangeTests() {
    selection = getSelection();
    testDiv = document.querySelector("#test");
    if (testDiv) {
        testDiv.parentNode.removeChild(testDiv);
    }
    testDiv = document.createElement("div");
    testDiv.id = "test";
    document.body.insertBefore(testDiv, document.body.firstChild);
    // Test some diacritics, to make sure browsers are using code units here
    // and not something like grapheme clusters.
    testDiv.innerHTML = "<p id=a>A&#x308;b&#x308;c&#x308;d&#x308;e&#x308;f&#x308;g&#x308;h&#x308;\n"
        + "<p id=b style=display:none>Ijklmnop\n"
        + "<p id=c>Qrstuvwx"
        + "<p id=d style=display:none>Yzabcdef"
        + "<p id=e style=display:none>Ghijklmn";
    paras = testDiv.querySelectorAll("p");

    detachedDiv = document.createElement("div");
    detachedPara1 = document.createElement("p");
    detachedPara1.appendChild(document.createTextNode("Opqrstuv"));
    detachedPara2 = document.createElement("p");
    detachedPara2.appendChild(document.createTextNode("Wxyzabcd"));
    detachedDiv.appendChild(detachedPara1);
    detachedDiv.appendChild(detachedPara2);

    // Opera doesn't automatically create a doctype for a new HTML document,
    // contrary to spec.  It also doesn't let you add doctypes to documents
    // after the fact through any means I've tried.  So foreignDoc in Opera
    // will have no doctype, foreignDoctype will be null, and Opera will fail
    // some tests somewhat mysteriously as a result.
    foreignDoc = document.implementation.createHTMLDocument("");
    foreignPara1 = foreignDoc.createElement("p");
    foreignPara1.appendChild(foreignDoc.createTextNode("Efghijkl"));
    foreignPara2 = foreignDoc.createElement("p");
    foreignPara2.appendChild(foreignDoc.createTextNode("Mnopqrst"));
    foreignDoc.body.appendChild(foreignPara1);
    foreignDoc.body.appendChild(foreignPara2);

    // Now we get to do really silly stuff, which nobody in the universe is
    // ever going to actually do, but the spec defines behavior, so too bad.
    // Testing is fun!
    xmlDoctype = document.implementation.createDocumentType("qorflesnorf", "abcde", "x\"'y");
    xmlDoc = document.implementation.createDocument(null, null, xmlDoctype);
    detachedXmlElement = xmlDoc.createElement("everyone-hates-hyphenated-element-names");
    detachedTextNode = document.createTextNode("Uvwxyzab");
    detachedForeignTextNode = foreignDoc.createTextNode("Cdefghij");
    detachedXmlTextNode = xmlDoc.createTextNode("Klmnopqr");
    // PIs only exist in XML documents, so don't bother with document or
    // foreignDoc.
    detachedProcessingInstruction = xmlDoc.createProcessingInstruction("whippoorwill", "chirp chirp chirp");
    detachedComment = document.createComment("Stuvwxyz");
    // Hurrah, we finally got to "z" at the end!
    detachedForeignComment = foreignDoc.createComment("אריה יהודה");
    detachedXmlComment = xmlDoc.createComment("בן חיים אליעזר");

    // We should also test with document fragments that actually contain stuff
    // . . . but, maybe later.
    docfrag = document.createDocumentFragment();
    foreignDocfrag = foreignDoc.createDocumentFragment();
    xmlDocfrag = xmlDoc.createDocumentFragment();

    xmlElement = xmlDoc.createElement("igiveuponcreativenames");
    xmlTextNode = xmlDoc.createTextNode("do re mi fa so la ti");
    xmlElement.appendChild(xmlTextNode);
    processingInstruction = xmlDoc.createProcessingInstruction("somePI", 'Did you know that ":syn sync fromstart" is very useful when using vim to edit large amounts of JavaScript embedded in HTML?');
    xmlDoc.appendChild(xmlElement);
    xmlDoc.appendChild(processingInstruction);
    xmlComment = xmlDoc.createComment("I maliciously created a comment that will break incautious XML serializers, but Firefox threw an exception, so all I got was this lousy T-shirt");
    xmlDoc.appendChild(xmlComment);

    comment = document.createComment("Alphabet soup?");
    testDiv.appendChild(comment);

    foreignComment = foreignDoc.createComment('"Commenter" and "commentator" mean different things.  I\'ve seen non-native speakers trip up on this.');
    foreignDoc.appendChild(foreignComment);
    foreignTextNode = foreignDoc.createTextNode("I admit that I harbor doubts about whether we really need so many things to test, but it's too late to stop now.");
    foreignDoc.body.appendChild(foreignTextNode);

    doctype = document.doctype;
    foreignDoctype = foreignDoc.doctype;

    // Chromium project has a limitation of text file size, and it is applied to
    // test result documents too.  Generating tests with testRanges or
    // testPoints can exceed the limitation easily.  Some tests were split into
    // multiple files such as addRange-NN.html.  If you add more ranges, points,
    // or tests, a Chromium project member might split affected tests.
    //
    // In selection/, a rough estimation of the limit is 4,000 test() functions
    // per a file.
    testRanges = [
        // Various ranges within the text node children of different
        // paragraphs.  All should be valid.
        "[paras[0].firstChild, 0, paras[0].firstChild, 0]",
        "[paras[0].firstChild, 0, paras[0].firstChild, 1]",
        "[paras[0].firstChild, 2, paras[0].firstChild, 8]",
        "[paras[0].firstChild, 2, paras[0].firstChild, 9]",
        "[paras[1].firstChild, 0, paras[1].firstChild, 0]",
        "[paras[1].firstChild, 0, paras[1].firstChild, 1]",
        "[paras[1].firstChild, 2, paras[1].firstChild, 8]",
        "[paras[1].firstChild, 2, paras[1].firstChild, 9]",
        "[detachedPara1.firstChild, 0, detachedPara1.firstChild, 0]",
        "[detachedPara1.firstChild, 0, detachedPara1.firstChild, 1]",
        "[detachedPara1.firstChild, 2, detachedPara1.firstChild, 8]",
        "[foreignPara1.firstChild, 0, foreignPara1.firstChild, 0]",
        "[foreignPara1.firstChild, 0, foreignPara1.firstChild, 1]",
        "[foreignPara1.firstChild, 2, foreignPara1.firstChild, 8]",
        // Now try testing some elements, not just text nodes.
        "[document.documentElement, 0, document.documentElement, 1]",
        "[document.documentElement, 0, document.documentElement, 2]",
        "[document.documentElement, 1, document.documentElement, 2]",
        "[document.head, 1, document.head, 1]",
        "[document.body, 0, document.body, 1]",
        "[foreignDoc.documentElement, 0, foreignDoc.documentElement, 1]",
        "[foreignDoc.head, 1, foreignDoc.head, 1]",
        "[foreignDoc.body, 0, foreignDoc.body, 0]",
        "[paras[0], 0, paras[0], 0]",
        "[paras[0], 0, paras[0], 1]",
        "[detachedPara1, 0, detachedPara1, 0]",
        "[detachedPara1, 0, detachedPara1, 1]",
        // Now try some ranges that span elements.
        "[paras[0].firstChild, 0, paras[1].firstChild, 0]",
        "[paras[0].firstChild, 0, paras[1].firstChild, 8]",
        "[paras[0].firstChild, 3, paras[3], 1]",
        // How about something that spans a node and its descendant?
        "[paras[0], 0, paras[0].firstChild, 7]",
        "[testDiv, 2, paras[4], 1]",
        "[testDiv, 1, paras[2].firstChild, 5]",
        "[document.documentElement, 1, document.body, 0]",
        "[foreignDoc.documentElement, 1, foreignDoc.body, 0]",
        // Then a few more interesting things just for good measure.
        "[document, 0, document, 1]",
        "[document, 0, document, 2]",
        "[document, 1, document, 2]",
        "[testDiv, 0, comment, 5]",
        "[paras[2].firstChild, 4, comment, 2]",
        "[paras[3], 1, comment, 8]",
        "[foreignDoc, 0, foreignDoc, 0]",
        "[foreignDoc, 1, foreignComment, 2]",
        "[foreignDoc.body, 0, foreignTextNode, 36]",
        "[xmlDoc, 0, xmlDoc, 0]",
        // Opera 11 crashes if you extractContents() a range that ends at offset
        // zero in a comment.  Comment out this line to run the tests successfully.
        "[xmlDoc, 1, xmlComment, 0]",
        "[detachedTextNode, 0, detachedTextNode, 8]",
        "[detachedForeignTextNode, 7, detachedForeignTextNode, 7]",
        "[detachedForeignTextNode, 0, detachedForeignTextNode, 8]",
        "[detachedXmlTextNode, 7, detachedXmlTextNode, 7]",
        "[detachedXmlTextNode, 0, detachedXmlTextNode, 8]",
        "[detachedComment, 3, detachedComment, 4]",
        "[detachedComment, 5, detachedComment, 5]",
        "[detachedForeignComment, 0, detachedForeignComment, 1]",
        "[detachedForeignComment, 4, detachedForeignComment, 4]",
        "[detachedXmlComment, 2, detachedXmlComment, 6]",
        "[docfrag, 0, docfrag, 0]",
        "[foreignDocfrag, 0, foreignDocfrag, 0]",
        "[xmlDocfrag, 0, xmlDocfrag, 0]",
    ];

    testPoints = [
        // Various positions within the page, some invalid.  Remember that
        // paras[0] is visible, and paras[1] is display: none.
        "[paras[0].firstChild, -1]",
        "[paras[0].firstChild, 0]",
        "[paras[0].firstChild, 1]",
        "[paras[0].firstChild, 2]",
        "[paras[0].firstChild, 8]",
        "[paras[0].firstChild, 9]",
        "[paras[0].firstChild, 10]",
        "[paras[0].firstChild, 65535]",
        "[paras[1].firstChild, -1]",
        "[paras[1].firstChild, 0]",
        "[paras[1].firstChild, 1]",
        "[paras[1].firstChild, 2]",
        "[paras[1].firstChild, 8]",
        "[paras[1].firstChild, 9]",
        "[paras[1].firstChild, 10]",
        "[paras[1].firstChild, 65535]",
        "[detachedPara1.firstChild, 0]",
        "[detachedPara1.firstChild, 1]",
        "[detachedPara1.firstChild, 8]",
        "[detachedPara1.firstChild, 9]",
        "[foreignPara1.firstChild, 0]",
        "[foreignPara1.firstChild, 1]",
        "[foreignPara1.firstChild, 8]",
        "[foreignPara1.firstChild, 9]",
        // Now try testing some elements, not just text nodes.
        "[document.documentElement, -1]",
        "[document.documentElement, 0]",
        "[document.documentElement, 1]",
        "[document.documentElement, 2]",
        "[document.documentElement, 7]",
        "[document.head, 1]",
        "[document.body, 3]",
        "[foreignDoc.documentElement, 0]",
        "[foreignDoc.documentElement, 1]",
        "[foreignDoc.head, 0]",
        "[foreignDoc.body, 1]",
        "[paras[0], 0]",
        "[paras[0], 1]",
        "[paras[0], 2]",
        "[paras[1], 0]",
        "[paras[1], 1]",
        "[paras[1], 2]",
        "[detachedPara1, 0]",
        "[detachedPara1, 1]",
        "[testDiv, 0]",
        "[testDiv, 3]",
        // Then a few more interesting things just for good measure.
        "[document, -1]",
        "[document, 0]",
        "[document, 1]",
        "[document, 2]",
        "[document, 3]",
        "[comment, -1]",
        "[comment, 0]",
        "[comment, 4]",
        "[comment, 96]",
        "[foreignDoc, 0]",
        "[foreignDoc, 1]",
        "[foreignComment, 2]",
        "[foreignTextNode, 0]",
        "[foreignTextNode, 36]",
        "[xmlDoc, -1]",
        "[xmlDoc, 0]",
        "[xmlDoc, 1]",
        "[xmlDoc, 5]",
        "[xmlComment, 0]",
        "[xmlComment, 4]",
        "[processingInstruction, 0]",
        "[processingInstruction, 5]",
        "[processingInstruction, 9]",
        "[detachedTextNode, 0]",
        "[detachedTextNode, 8]",
        "[detachedForeignTextNode, 0]",
        "[detachedForeignTextNode, 8]",
        "[detachedXmlTextNode, 0]",
        "[detachedXmlTextNode, 8]",
        "[detachedProcessingInstruction, 12]",
        "[detachedComment, 3]",
        "[detachedComment, 5]",
        "[detachedForeignComment, 0]",
        "[detachedForeignComment, 4]",
        "[detachedXmlComment, 2]",
        "[docfrag, 0]",
        "[foreignDocfrag, 0]",
        "[xmlDocfrag, 0]",
        "[doctype, 0]",
        "[doctype, -17]",
        "[doctype, 1]",
        "[foreignDoctype, 0]",
        "[xmlDoctype, 0]",
    ];

    testNodes = [
        "paras[0]",
        "paras[0].firstChild",
        "paras[1]",
        "paras[1].firstChild",
        "foreignPara1",
        "foreignPara1.firstChild",
        "detachedPara1",
        "detachedPara1.firstChild",
        "detachedPara1",
        "detachedPara1.firstChild",
        "testDiv",
        "document",
        "detachedDiv",
        "detachedPara2",
        "foreignDoc",
        "foreignPara2",
        "xmlDoc",
        "xmlElement",
        "detachedXmlElement",
        "detachedTextNode",
        "foreignTextNode",
        "detachedForeignTextNode",
        "xmlTextNode",
        "detachedXmlTextNode",
        "processingInstruction",
        "detachedProcessingInstruction",
        "comment",
        "detachedComment",
        "foreignComment",
        "detachedForeignComment",
        "xmlComment",
        "detachedXmlComment",
        "docfrag",
        "foreignDocfrag",
        "xmlDocfrag",
        "doctype",
        "foreignDoctype",
        "xmlDoctype",
    ];
}
if ("setup" in window) {
    setup(setupRangeTests);
} else {
    // Presumably we're running from within an iframe or something
    setupRangeTests();
}

/**
 * Return the length of a node as specified in DOM Range.
 */
function getNodeLength(node) {
    if (node.nodeType == Node.DOCUMENT_TYPE_NODE) {
        return 0;
    }
    if (node.nodeType == Node.TEXT_NODE || node.nodeType == Node.PROCESSING_INSTRUCTION_NODE || node.nodeType == Node.COMMENT_NODE) {
        return node.length;
    }
    return node.childNodes.length;
}

/**
 * Returns the furthest ancestor of a Node as defined by the spec.
 */
function furthestAncestor(node) {
    var root = node;
    while (root.parentNode != null) {
        root = root.parentNode;
    }
    return root;
}

/**
 * "The ancestor containers of a Node are the Node itself and all its
 * ancestors."
 *
 * Is node1 an ancestor container of node2?
 */
function isAncestorContainer(node1, node2) {
    return node1 == node2 ||
        (node2.compareDocumentPosition(node1) & Node.DOCUMENT_POSITION_CONTAINS);
}

/**
 * Returns the first Node that's after node in tree order, or null if node is
 * the last Node.
 */
function nextNode(node) {
    if (node.hasChildNodes()) {
        return node.firstChild;
    }
    return nextNodeDescendants(node);
}

/**
 * Returns the last Node that's before node in tree order, or null if node is
 * the first Node.
 */
function previousNode(node) {
    if (node.previousSibling) {
        node = node.previousSibling;
        while (node.hasChildNodes()) {
            node = node.lastChild;
        }
        return node;
    }
    return node.parentNode;
}

/**
 * Returns the next Node that's after node and all its descendants in tree
 * order, or null if node is the last Node or an ancestor of it.
 */
function nextNodeDescendants(node) {
    while (node && !node.nextSibling) {
        node = node.parentNode;
    }
    if (!node) {
        return null;
    }
    return node.nextSibling;
}

/**
 * Returns the ownerDocument of the Node, or the Node itself if it's a
 * Document.
 */
function ownerDocument(node) {
    return node.nodeType == Node.DOCUMENT_NODE
        ? node
        : node.ownerDocument;
}

/**
 * Returns true if ancestor is an ancestor of descendant, false otherwise.
 */
function isAncestor(ancestor, descendant) {
    if (!ancestor || !descendant) {
        return false;
    }
    while (descendant && descendant != ancestor) {
        descendant = descendant.parentNode;
    }
    return descendant == ancestor;
}

/**
 * Returns true if descendant is a descendant of ancestor, false otherwise.
 */
function isDescendant(descendant, ancestor) {
    return isAncestor(ancestor, descendant);
}

/**
 * The position of two boundary points relative to one another, as defined by
 * the spec.
 */
function getPosition(nodeA, offsetA, nodeB, offsetB) {
    // "If node A is the same as node B, return equal if offset A equals offset
    // B, before if offset A is less than offset B, and after if offset A is
    // greater than offset B."
    if (nodeA == nodeB) {
        if (offsetA == offsetB) {
            return "equal";
        }
        if (offsetA < offsetB) {
            return "before";
        }
        if (offsetA > offsetB) {
            return "after";
        }
    }

    // "If node A is after node B in tree order, compute the position of (node
    // B, offset B) relative to (node A, offset A). If it is before, return
    // after. If it is after, return before."
    if (nodeB.compareDocumentPosition(nodeA) & Node.DOCUMENT_POSITION_FOLLOWING) {
        var pos = getPosition(nodeB, offsetB, nodeA, offsetA);
        if (pos == "before") {
            return "after";
        }
        if (pos == "after") {
            return "before";
        }
    }

    // "If node A is an ancestor of node B:"
    if (nodeB.compareDocumentPosition(nodeA) & Node.DOCUMENT_POSITION_CONTAINS) {
        // "Let child equal node B."
        var child = nodeB;

        // "While child is not a child of node A, set child to its parent."
        while (child.parentNode != nodeA) {
            child = child.parentNode;
        }

        // "If the index of child is less than offset A, return after."
        if (indexOf(child) < offsetA) {
            return "after";
        }
    }

    // "Return before."
    return "before";
}

/**
 * "contained" as defined by DOM Range: "A Node node is contained in a range
 * range if node's furthest ancestor is the same as range's root, and (node, 0)
 * is after range's start, and (node, length of node) is before range's end."
 */
function isContained(node, range) {
    var pos1 = getPosition(node, 0, range.startContainer, range.startOffset);
    var pos2 = getPosition(node, getNodeLength(node), range.endContainer, range.endOffset);

    return furthestAncestor(node) == furthestAncestor(range.startContainer)
        && pos1 == "after"
        && pos2 == "before";
}

/**
 * "partially contained" as defined by DOM Range: "A Node is partially
 * contained in a range if it is an ancestor container of the range's start but
 * not its end, or vice versa."
 */
function isPartiallyContained(node, range) {
    var cond1 = isAncestorContainer(node, range.startContainer);
    var cond2 = isAncestorContainer(node, range.endContainer);
    return (cond1 && !cond2) || (cond2 && !cond1);
}

/**
 * Index of a node as defined by the spec.
 */
function indexOf(node) {
    if (!node.parentNode) {
        // No preceding sibling nodes, right?
        return 0;
    }
    var i = 0;
    while (node != node.parentNode.childNodes[i]) {
        i++;
    }
    return i;
}

/**
 * extractContents() implementation, following the spec.  If an exception is
 * supposed to be thrown, will return a string with the name (e.g.,
 * "HIERARCHY_REQUEST_ERR") instead of a document fragment.  It might also
 * return an arbitrary human-readable string if a condition is hit that implies
 * a spec bug.
 */
function myExtractContents(range) {
    // "If the context object's detached flag is set, raise an
    // INVALID_STATE_ERR exception and abort these steps."
    try {
        range.collapsed;
    } catch (e) {
        return "INVALID_STATE_ERR";
    }

    // "Let frag be a new DocumentFragment whose ownerDocument is the same as
    // the ownerDocument of the context object's start node."
    var ownerDoc = range.startContainer.nodeType == Node.DOCUMENT_NODE
        ? range.startContainer
        : range.startContainer.ownerDocument;
    var frag = ownerDoc.createDocumentFragment();

    // "If the context object's start and end are the same, abort this method,
    // returning frag."
    if (range.startContainer == range.endContainer
    && range.startOffset == range.endOffset) {
        return frag;
    }

    // "Let original start node, original start offset, original end node, and
    // original end offset be the context object's start and end nodes and
    // offsets, respectively."
    var originalStartNode = range.startContainer;
    var originalStartOffset = range.startOffset;
    var originalEndNode = range.endContainer;
    var originalEndOffset = range.endOffset;

    // "If original start node and original end node are the same, and they are
    // a Text or Comment node:"
    if (range.startContainer == range.endContainer
    && (range.startContainer.nodeType == Node.TEXT_NODE
    || range.startContainer.nodeType == Node.COMMENT_NODE)) {
        // "Let clone be the result of calling cloneNode(false) on original
        // start node."
        var clone = originalStartNode.cloneNode(false);

        // "Set the data of clone to the result of calling
        // substringData(original start offset, original end offset − original
        // start offset) on original start node."
        clone.data = originalStartNode.substringData(originalStartOffset,
            originalEndOffset - originalStartOffset);

        // "Append clone as the last child of frag."
        frag.appendChild(clone);

        // "Call deleteData(original start offset, original end offset −
        // original start offset) on original start node."
        originalStartNode.deleteData(originalStartOffset,
            originalEndOffset - originalStartOffset);

        // "Abort this method, returning frag."
        return frag;
    }

    // "Let common ancestor equal original start node."
    var commonAncestor = originalStartNode;

    // "While common ancestor is not an ancestor container of original end
    // node, set common ancestor to its own parent."
    while (!isAncestorContainer(commonAncestor, originalEndNode)) {
        commonAncestor = commonAncestor.parentNode;
    }

    // "If original start node is an ancestor container of original end node,
    // let first partially contained child be null."
    var firstPartiallyContainedChild;
    if (isAncestorContainer(originalStartNode, originalEndNode)) {
        firstPartiallyContainedChild = null;
    // "Otherwise, let first partially contained child be the first child of
    // common ancestor that is partially contained in the context object."
    } else {
        for (var i = 0; i < commonAncestor.childNodes.length; i++) {
            if (isPartiallyContained(commonAncestor.childNodes[i], range)) {
                firstPartiallyContainedChild = commonAncestor.childNodes[i];
                break;
            }
        }
        if (!firstPartiallyContainedChild) {
            throw "Spec bug: no first partially contained child!";
        }
    }

    // "If original end node is an ancestor container of original start node,
    // let last partially contained child be null."
    var lastPartiallyContainedChild;
    if (isAncestorContainer(originalEndNode, originalStartNode)) {
        lastPartiallyContainedChild = null;
    // "Otherwise, let last partially contained child be the last child of
    // common ancestor that is partially contained in the context object."
    } else {
        for (var i = commonAncestor.childNodes.length - 1; i >= 0; i--) {
            if (isPartiallyContained(commonAncestor.childNodes[i], range)) {
                lastPartiallyContainedChild = commonAncestor.childNodes[i];
                break;
            }
        }
        if (!lastPartiallyContainedChild) {
            throw "Spec bug: no last partially contained child!";
        }
    }

    // "Let contained children be a list of all children of common ancestor
    // that are contained in the context object, in tree order."
    //
    // "If any member of contained children is a DocumentType, raise a
    // HIERARCHY_REQUEST_ERR exception and abort these steps."
    var containedChildren = [];
    for (var i = 0; i < commonAncestor.childNodes.length; i++) {
        if (isContained(commonAncestor.childNodes[i], range)) {
            if (commonAncestor.childNodes[i].nodeType
            == Node.DOCUMENT_TYPE_NODE) {
                return "HIERARCHY_REQUEST_ERR";
            }
            containedChildren.push(commonAncestor.childNodes[i]);
        }
    }

    // "If original start node is an ancestor container of original end node,
    // set new node to original start node and new offset to original start
    // offset."
    var newNode, newOffset;
    if (isAncestorContainer(originalStartNode, originalEndNode)) {
        newNode = originalStartNode;
        newOffset = originalStartOffset;
    // "Otherwise:"
    } else {
        // "Let reference node equal original start node."
        var referenceNode = originalStartNode;

        // "While reference node's parent is not null and is not an ancestor
        // container of original end node, set reference node to its parent."
        while (referenceNode.parentNode
        && !isAncestorContainer(referenceNode.parentNode, originalEndNode)) {
            referenceNode = referenceNode.parentNode;
        }

        // "Set new node to the parent of reference node, and new offset to one
        // plus the index of reference node."
        newNode = referenceNode.parentNode;
        newOffset = 1 + indexOf(referenceNode);
    }

    // "If first partially contained child is a Text or Comment node:"
    if (firstPartiallyContainedChild
    && (firstPartiallyContainedChild.nodeType == Node.TEXT_NODE
    || firstPartiallyContainedChild.nodeType == Node.COMMENT_NODE)) {
        // "Let clone be the result of calling cloneNode(false) on original
        // start node."
        var clone = originalStartNode.cloneNode(false);

        // "Set the data of clone to the result of calling substringData() on
        // original start node, with original start offset as the first
        // argument and (length of original start node − original start offset)
        // as the second."
        clone.data = originalStartNode.substringData(originalStartOffset,
            getNodeLength(originalStartNode) - originalStartOffset);

        // "Append clone as the last child of frag."
        frag.appendChild(clone);

        // "Call deleteData() on original start node, with original start
        // offset as the first argument and (length of original start node −
        // original start offset) as the second."
        originalStartNode.deleteData(originalStartOffset,
            getNodeLength(originalStartNode) - originalStartOffset);
    // "Otherwise, if first partially contained child is not null:"
    } else if (firstPartiallyContainedChild) {
        // "Let clone be the result of calling cloneNode(false) on first
        // partially contained child."
        var clone = firstPartiallyContainedChild.cloneNode(false);

        // "Append clone as the last child of frag."
        frag.appendChild(clone);

        // "Let subrange be a new Range whose start is (original start node,
        // original start offset) and whose end is (first partially contained
        // child, length of first partially contained child)."
        var subrange = ownerDoc.createRange();
        subrange.setStart(originalStartNode, originalStartOffset);
        subrange.setEnd(firstPartiallyContainedChild,
            getNodeLength(firstPartiallyContainedChild));

        // "Let subfrag be the result of calling extractContents() on
        // subrange."
        var subfrag = myExtractContents(subrange);

        // "For each child of subfrag, in order, append that child to clone as
        // its last child."
        for (var i = 0; i < subfrag.childNodes.length; i++) {
            clone.appendChild(subfrag.childNodes[i]);
        }
    }

    // "For each contained child in contained children, append contained child
    // as the last child of frag."
    for (var i = 0; i < containedChildren.length; i++) {
        frag.appendChild(containedChildren[i]);
    }

    // "If last partially contained child is a Text or Comment node:"
    if (lastPartiallyContainedChild
    && (lastPartiallyContainedChild.nodeType == Node.TEXT_NODE
    || lastPartiallyContainedChild.nodeType == Node.COMMENT_NODE)) {
        // "Let clone be the result of calling cloneNode(false) on original
        // end node."
        var clone = originalEndNode.cloneNode(false);

        // "Set the data of clone to the result of calling substringData(0,
        // original end offset) on original end node."
        clone.data = originalEndNode.substringData(0, originalEndOffset);

        // "Append clone as the last child of frag."
        frag.appendChild(clone);

        // "Call deleteData(0, original end offset) on original end node."
        originalEndNode.deleteData(0, originalEndOffset);
    // "Otherwise, if last partially contained child is not null:"
    } else if (lastPartiallyContainedChild) {
        // "Let clone be the result of calling cloneNode(false) on last
        // partially contained child."
        var clone = lastPartiallyContainedChild.cloneNode(false);

        // "Append clone as the last child of frag."
        frag.appendChild(clone);

        // "Let subrange be a new Range whose start is (last partially
        // contained child, 0) and whose end is (original end node, original
        // end offset)."
        var subrange = ownerDoc.createRange();
        subrange.setStart(lastPartiallyContainedChild, 0);
        subrange.setEnd(originalEndNode, originalEndOffset);

        // "Let subfrag be the result of calling extractContents() on
        // subrange."
        var subfrag = myExtractContents(subrange);

        // "For each child of subfrag, in order, append that child to clone as
        // its last child."
        for (var i = 0; i < subfrag.childNodes.length; i++) {
            clone.appendChild(subfrag.childNodes[i]);
        }
    }

    // "Set the context object's start and end to (new node, new offset)."
    range.setStart(newNode, newOffset);
    range.setEnd(newNode, newOffset);

    // "Return frag."
    return frag;
}

/**
 * insertNode() implementation, following the spec.  If an exception is
 * supposed to be thrown, will return a string with the name (e.g.,
 * "HIERARCHY_REQUEST_ERR") instead of a document fragment.  It might also
 * return an arbitrary human-readable string if a condition is hit that implies
 * a spec bug.
 */
function myInsertNode(range, newNode) {
    // "If the context object's detached flag is set, raise an
    // INVALID_STATE_ERR exception and abort these steps."
    //
    // Assume that if accessing collapsed throws, it's detached.
    try {
        range.collapsed;
    } catch (e) {
        return "INVALID_STATE_ERR";
    }

    // "If the context object's start node is a Text or Comment node and its
    // parent is null, raise an HIERARCHY_REQUEST_ERR exception and abort these
    // steps."
    if ((range.startContainer.nodeType == Node.TEXT_NODE
    || range.startContainer.nodeType == Node.COMMENT_NODE)
    && !range.startContainer.parentNode) {
        return "HIERARCHY_REQUEST_ERR";
    }

    // "If the context object's start node is a Text node, run splitText() on
    // it with the context object's start offset as its argument, and let
    // reference node be the result."
    var referenceNode;
    if (range.startContainer.nodeType == Node.TEXT_NODE) {
        // We aren't testing how ranges vary under mutations, and browsers vary
        // in how they mutate for splitText, so let's just force the correct
        // way.
        var start = [range.startContainer, range.startOffset];
        var end = [range.endContainer, range.endOffset];

        referenceNode = range.startContainer.splitText(range.startOffset);

        if (start[0] == end[0]
        && end[1] > start[1]) {
            end[0] = referenceNode;
            end[1] -= start[1];
        } else if (end[0] == start[0].parentNode
        && end[1] > indexOf(referenceNode)) {
            end[1]++;
        }
        range.setStart(start[0], start[1]);
        range.setEnd(end[0], end[1]);
    // "Otherwise, if the context object's start node is a Comment, let
    // reference node be the context object's start node."
    } else if (range.startContainer.nodeType == Node.COMMENT_NODE) {
        referenceNode = range.startContainer;
    // "Otherwise, let reference node be the child of the context object's
    // start node with index equal to the context object's start offset, or
    // null if there is no such child."
    } else {
        referenceNode = range.startContainer.childNodes[range.startOffset];
        if (typeof referenceNode == "undefined") {
            referenceNode = null;
        }
    }

    // "If reference node is null, let parent node be the context object's
    // start node."
    var parentNode;
    if (!referenceNode) {
        parentNode = range.startContainer;
    // "Otherwise, let parent node be the parent of reference node."
    } else {
        parentNode = referenceNode.parentNode;
    }

    // "Call insertBefore(newNode, reference node) on parent node, re-raising
    // any exceptions that call raised."
    try {
        parentNode.insertBefore(newNode, referenceNode);
    } catch (e) {
        return getDomExceptionName(e);
    }
}

/**
 * Asserts that two nodes are equal, in the sense of isEqualNode().  If they
 * aren't, tries to print a relatively informative reason why not.  TODO: Move
 * this to testharness.js?
 */
function assertNodesEqual(actual, expected, msg) {
    if (!actual.isEqualNode(expected)) {
        msg = "Actual and expected mismatch for " + msg + ".  ";

        while (actual && expected) {
            assert_true(actual.nodeType === expected.nodeType
                && actual.nodeName === expected.nodeName
                && actual.nodeValue === expected.nodeValue
                && actual.childNodes.length === expected.childNodes.length,
                "First differing node: expected " + format_value(expected)
                + ", got " + format_value(actual));
            actual = nextNode(actual);
            expected = nextNode(expected);
        }

        assert_unreached("DOMs were not equal but we couldn't figure out why");
    }
}

/**
 * Given a DOMException, return the name (e.g., "HIERARCHY_REQUEST_ERR").  In
 * theory this should be just e.name, but in practice it's not.  So I could
 * legitimately just return e.name, but then every engine but WebKit would fail
 * every test, since no one seems to care much for standardizing DOMExceptions.
 * Instead I mangle it to account for browser bugs, so as not to fail
 * insertNode() tests (for instance) for insertBefore() bugs.  Of course, a
 * standards-compliant browser will work right in any event.
 *
 * If the exception has no string property called "name" or "message", we just
 * re-throw it.
 */
function getDomExceptionName(e) {
    if (typeof e.name == "string"
    && /^[A-Z_]+_ERR$/.test(e.name)) {
        // Either following the standard, or prefixing NS_ERROR_DOM (I'm
        // looking at you, Gecko).
        return e.name.replace(/^NS_ERROR_DOM_/, "");
    }

    if (typeof e.message == "string"
    && /^[A-Z_]+_ERR$/.test(e.message)) {
        // Opera
        return e.message;
    }

    if (typeof e.message == "string"
    && /^DOM Exception:/.test(e.message)) {
        // IE
        return /[A-Z_]+_ERR/.exec(e.message)[0];
    }

    throw e;
}

/**
 * Given an array of endpoint data [start container, start offset, end
 * container, end offset], returns a Range with those endpoints.
 */
function rangeFromEndpoints(endpoints) {
    // If we just use document instead of the ownerDocument of endpoints[0],
    // WebKit will throw on setStart/setEnd.  This is a WebKit bug, but it's in
    // range, not selection, so we don't want to fail anything for it.
    var range = ownerDocument(endpoints[0]).createRange();
    range.setStart(endpoints[0], endpoints[1]);
    range.setEnd(endpoints[2], endpoints[3]);
    return range;
}

/**
 * Given an array of endpoint data [start container, start offset, end
 * container, end offset], sets the selection to have those endpoints.  Uses
 * addRange, so the range will be forwards.  Accepts an empty array for
 * endpoints, in which case the selection will just be emptied.
 */
function setSelectionForwards(endpoints) {
    selection.removeAllRanges();
    if (endpoints.length) {
        selection.addRange(rangeFromEndpoints(endpoints));
    }
}

/**
 * Given an array of endpoint data [start container, start offset, end
 * container, end offset], sets the selection to have those endpoints, with the
 * direction backwards.  Uses extend, so it will throw in IE.  Accepts an empty
 * array for endpoints, in which case the selection will just be emptied.
 */
function setSelectionBackwards(endpoints) {
    selection.removeAllRanges();
    if (endpoints.length) {
        selection.collapse(endpoints[2], endpoints[3]);
        selection.extend(endpoints[0], endpoints[1]);
    }
}

/**
 * Verify that the specified func doesn't change the selection.
 * This function should be used in testharness tests.
 */
function assertSelectionNoChange(func) {
    var originalCount = selection.rangeCount;
    var originalRange = originalCount == 0 ? null : selection.getRangeAt(0);
    var originalAnchorNode = selection.anchorNode;
    var originalAnchorOffset = selection.anchorOffset;
    var originalFocusNode = selection.focusNode;
    var originalFocusOffset = selection.focusOffset;

    func();

    assert_equals(selection.rangeCount, originalCount,
        "The operation should not add Range");
    assert_equals(selection.anchorNode, originalAnchorNode,
        "The operation should not update anchorNode");
    assert_equals(selection.anchorOffset, originalAnchorOffset,
        "The operation should not update anchorOffset");
    assert_equals(selection.focusNode, originalFocusNode,
        "The operation should not update focusNode");
    assert_equals(selection.focusOffset, originalFocusOffset,
        "The operation should not update focusOffset");
    if (originalCount < 1)
        return;
    assert_equals(selection.getRangeAt(0), originalRange,
         "The operation should not replace a registered Range");
}

/**
 * Check if the specified node can be selectable with window.getSelection()
 * methods.
 */
function isSelectableNode(node) {
    if (!node)
        return false;
    if (node.nodeType == Node.DOCUMENT_TYPE_NODE)
        return false;
    return document.contains(node);
}
