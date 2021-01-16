"use strict";

// TODO: extend `EditorTestUtils` in editing/include/edit-test-utils.mjs

const kBackspaceKey = "\uE003";
const kDeleteKey = "\uE017";
const kArrowRight = "\uE014";
const kArrowLeft = "\uE012";
const kShift = "\uE008";
const kMeta = "\uE03d";
const kControl = "\uE009";
const kAlt = "\uE00A";
const kKeyA = "a";

const kImgSrc =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAEElEQVR42mNgaGD4D8YwBgAw9AX9Y9zBwwAAAABJRU5ErkJggg==";

let gSelection, gEditor, gBeforeinput, gInput;

function initializeTest(aInnerHTML) {
  function onBeforeinput(event) {
    // NOTE: Blink makes `getTargetRanges()` return empty range after
    //       propagation, but this test wants to check the result during
    //       propagation.  Therefore, we need to cache the result, but will
    //       assert if `getTargetRanges()` returns different ranges after
    //       checking the cached ranges.
    event.cachedRanges = event.getTargetRanges();
    gBeforeinput.push(event);
  }
  function onInput(event) {
    event.cachedRanges = event.getTargetRanges();
    gInput.push(event);
  }
  if (gEditor !== document.querySelector("div[contenteditable]")) {
    if (gEditor) {
      gEditor.isListeningToInputEvents = false;
      gEditor.removeEventListener("beforeinput", onBeforeinput);
      gEditor.removeEventListener("input", onInput);
    }
    gEditor = document.querySelector("div[contenteditable]");
  }
  gSelection = getSelection();
  gBeforeinput = [];
  gInput = [];
  if (!gEditor.isListeningToInputEvents) {
    gEditor.isListeningToInputEvents = true;
    gEditor.addEventListener("beforeinput", onBeforeinput);
    gEditor.addEventListener("input", onInput);
  }

  setupEditor(aInnerHTML);
  gBeforeinput = [];
  gInput = [];
}

function getArrayOfRangesDescription(arrayOfRanges) {
  if (arrayOfRanges === null) {
    return "null";
  }
  if (arrayOfRanges === undefined) {
    return "undefined";
  }
  if (!Array.isArray(arrayOfRanges)) {
    return "Unknown Object";
  }
  if (arrayOfRanges.length === 0) {
    return "[]";
  }
  let result = "[";
  for (let range of arrayOfRanges) {
    result += `{${getRangeDescription(range)}},`;
  }
  result += "]";
  return result;
}

function getRangeDescription(range) {
  function getNodeDescription(node) {
    if (!node) {
      return "null";
    }
    switch (node.nodeType) {
      case Node.TEXT_NODE:
      case Node.COMMENT_NODE:
      case Node.CDATA_SECTION_NODE:
        return `${node.nodeName} "${node.data}"`;
      case Node.ELEMENT_NODE:
        return `<${node.nodeName.toLowerCase()}>`;
      default:
        return `${node.nodeName}`;
    }
  }
  if (range === null) {
    return "null";
  }
  if (range === undefined) {
    return "undefined";
  }
  return range.startContainer == range.endContainer &&
    range.startOffset == range.endOffset
    ? `(${getNodeDescription(range.startContainer)}, ${range.startOffset})`
    : `(${getNodeDescription(range.startContainer)}, ${
        range.startOffset
      }) - (${getNodeDescription(range.endContainer)}, ${range.endOffset})`;
}

function sendDeleteKey(modifier) {
  if (!modifier) {
    return new test_driver.Actions()
      .keyDown(kDeleteKey)
      .keyUp(kDeleteKey)
      .send();
  }
  return new test_driver.Actions()
    .keyDown(modifier)
    .keyDown(kDeleteKey)
    .keyUp(kDeleteKey)
    .keyUp(modifier)
    .send();
}

function sendBackspaceKey(modifier) {
  if (!modifier) {
    return new test_driver.Actions()
      .keyDown(kBackspaceKey)
      .keyUp(kBackspaceKey)
      .send();
  }
  return new test_driver.Actions()
    .keyDown(modifier)
    .keyDown(kBackspaceKey)
    .keyUp(kBackspaceKey)
    .keyUp(modifier)
    .send();
}

function sendKeyA() {
  return new test_driver.Actions()
    .keyDown(kKeyA)
    .keyUp(kKeyA)
    .send();
}

function sendArrowLeftKey() {
  return new test_driver.Actions()
    .keyDown(kArrowLeft)
    .keyUp(kArrowLeft)
    .send();
}

function sendArrowRightKey() {
  return new test_driver.Actions()
    .keyDown(kArrowRight)
    .keyUp(kArrowRight)
    .send();
}

function checkGetTargetRangesOfBeforeinputOnDeleteSomething(expectedRanges) {
  assert_equals(
    gBeforeinput.length,
    1,
    "One beforeinput event should be fired if the key operation tries to delete something"
  );
  assert_true(
    Array.isArray(gBeforeinput[0].cachedRanges),
    "gBeforeinput[0].getTargetRanges() should return an array of StaticRange instances at least during propagation"
  );
  let arrayOfExpectedRanges = Array.isArray(expectedRanges)
    ? expectedRanges
    : [expectedRanges];
  // Before checking the length of array of ranges, we should check the given
  // range first because the ranges are more important than whether there are
  // redundant additional unexpected ranges.
  for (
    let i = 0;
    i <
    Math.max(arrayOfExpectedRanges.length, gBeforeinput[0].cachedRanges.length);
    i++
  ) {
    assert_equals(
      getRangeDescription(gBeforeinput[0].cachedRanges[i]),
      getRangeDescription(arrayOfExpectedRanges[i]),
      `gBeforeinput[0].getTargetRanges()[${i}] should return expected range (inputType is "${gBeforeinput[0].inputType}")`
    );
  }
  assert_equals(
    gBeforeinput[0].cachedRanges.length,
    arrayOfExpectedRanges.length,
    `getTargetRanges() of beforeinput event should return ${arrayOfExpectedRanges.length} ranges`
  );
}

function checkGetTargetRangesOfInputOnDeleteSomething() {
  assert_equals(
    gInput.length,
    1,
    "One input event should be fired if the key operation deletes something"
  );
  // https://github.com/w3c/input-events/issues/113
  assert_true(
    Array.isArray(gInput[0].cachedRanges),
    "gInput[0].getTargetRanges() should return an array of StaticRange instances at least during propagation"
  );
  assert_equals(
    gInput[0].cachedRanges.length,
    0,
    "gInput[0].getTargetRanges() should return empty array during propagation"
  );
}

function checkGetTargetRangesOfInputOnDoNothing() {
  assert_equals(
    gInput.length,
    0,
    "input event shouldn't be fired when the key operation does not cause modifying the DOM tree"
  );
}

function checkBeforeinputAndInputEventsOnNOOP() {
  assert_equals(
    gBeforeinput.length,
    0,
    "beforeinput event shouldn't be fired when the key operation does not cause modifying the DOM tree"
  );
  assert_equals(
    gInput.length,
    0,
    "input event shouldn't be fired when the key operation does not cause modifying the DOM tree"
  );
}

function checkEditorContentResultAsSubTest(
  expectedResult,
  description,
  options = {}
) {
  test(() => {
    if (Array.isArray(expectedResult)) {
      assert_in_array(
        options.ignoreWhiteSpaceDifference
          ? gEditor.innerHTML.replace(/&nbsp;/g, " ")
          : gEditor.innerHTML,
        expectedResult
      );
    } else {
      assert_equals(
        options.ignoreWhiteSpaceDifference
          ? gEditor.innerHTML.replace(/&nbsp;/g, " ")
          : gEditor.innerHTML,
        expectedResult
      );
    }
  }, `${description} - comparing innerHTML`);
}

// Similar to `setupDiv` in editing/include/tests.js, this method sets
// innerHTML value of gEditor, and sets multiple selection ranges specified
// with the markers.
// - `[` specifies start boundary in a text node
// - `{` specifies start boundary before a node
// - `]` specifies end boundary in a text node
// - `}` specifies end boundary after a node
function setupEditor(innerHTMLWithRangeMarkers) {
  const startBoundaries = innerHTMLWithRangeMarkers.match(/\{|\[/g) || [];
  const endBoundaries = innerHTMLWithRangeMarkers.match(/\}|\]/g) || [];
  if (startBoundaries.length !== endBoundaries.length) {
    throw "Should match number of open/close markers";
  }

  gEditor.innerHTML = innerHTMLWithRangeMarkers;
  gEditor.focus();

  if (startBoundaries.length === 0) {
    // Don't remove the range for now since some tests may assume that
    // setting innerHTML does not remove all selection ranges.
    return;
  }

  function getNextRangeAndDeleteMarker(startNode) {
    function getNextLeafNode(node) {
      function inclusiveDeepestFirstChildNode(container) {
        while (container.firstChild) {
          container = container.firstChild;
        }
        return container;
      }
      if (node.hasChildNodes()) {
        return inclusiveDeepestFirstChildNode(node);
      }
      if (node.nextSibling) {
        return inclusiveDeepestFirstChildNode(node.nextSibling);
      }
      let nextSibling = (function nextSiblingOfAncestorElement(child) {
        for (
          let parent = child.parentElement;
          parent && parent != gEditor;
          parent = parent.parentElement
        ) {
          if (parent.nextSibling) {
            return parent.nextSibling;
          }
        }
        return null;
      })(node);
      if (!nextSibling) {
        return null;
      }
      return inclusiveDeepestFirstChildNode(nextSibling);
    }
    function scanMarkerInTextNode(textNode, offset) {
      return /[\{\[\]\}]/.exec(textNode.data.substr(offset));
    }
    let startMarker = (function scanNextStartMaker(
      startContainer,
      startOffset
    ) {
      function scanStartMakerInTextNode(textNode, offset) {
        let scanResult = scanMarkerInTextNode(textNode, offset);
        if (scanResult === null) {
          return null;
        }
        if (scanResult[0] === "}" || scanResult[0] === "]") {
          throw "An end marker is found before a start marker";
        }
        return {
          marker: scanResult[0],
          container: textNode,
          offset: scanResult.index + offset
        };
      }
      if (startContainer.nodeType === Node.TEXT_NODE) {
        let scanResult = scanStartMakerInTextNode(startContainer, startOffset);
        if (scanResult !== null) {
          return scanResult;
        }
      }
      let nextNode = startContainer;
      while ((nextNode = getNextLeafNode(nextNode))) {
        if (nextNode.nodeType === Node.TEXT_NODE) {
          let scanResult = scanStartMakerInTextNode(nextNode, 0);
          if (scanResult !== null) {
            return scanResult;
          }
          continue;
        }
      }
      return null;
    })(startNode, 0);
    if (startMarker === null) {
      return null;
    }
    let endMarker = (function scanNextEndMarker(startContainer, startOffset) {
      function scanEndMarkerInTextNode(textNode, offset) {
        let scanResult = scanMarkerInTextNode(textNode, offset);
        if (scanResult === null) {
          return null;
        }
        if (scanResult[0] === "{" || scanResult[0] === "[") {
          throw "A start marker is found before an end marker";
        }
        return {
          marker: scanResult[0],
          container: textNode,
          offset: scanResult.index + offset
        };
      }
      if (startContainer.nodeType === Node.TEXT_NODE) {
        let scanResult = scanEndMarkerInTextNode(startContainer, startOffset);
        if (scanResult !== null) {
          return scanResult;
        }
      }
      let nextNode = startContainer;
      while ((nextNode = getNextLeafNode(nextNode))) {
        if (nextNode.nodeType === Node.TEXT_NODE) {
          let scanResult = scanEndMarkerInTextNode(nextNode, 0);
          if (scanResult !== null) {
            return scanResult;
          }
          continue;
        }
      }
      return null;
    })(startMarker.container, startMarker.offset + 1);
    if (endMarker === null) {
      throw "Found an open marker, but not found corresponding close marker";
    }
    function indexOfContainer(container, child) {
      let offset = 0;
      for (let node = container.firstChild; node; node = node.nextSibling) {
        if (node == child) {
          return offset;
        }
        offset++;
      }
      throw "child must be a child node of container";
    }
    (function deleteFoundMarkers() {
      function removeNode(node) {
        let container = node.parentElement;
        let offset = indexOfContainer(container, node);
        node.remove();
        return { container, offset };
      }
      if (startMarker.container == endMarker.container) {
        // If the text node becomes empty, remove it and set collapsed range
        // to the position where there is the text node.
        if (startMarker.container.length === 2) {
          if (!/[\[\{][\]\}]/.test(startMarker.container.data)) {
            throw `Unexpected text node (data: "${startMarker.container.data}")`;
          }
          let { container, offset } = removeNode(startMarker.container);
          startMarker.container = endMarker.container = container;
          startMarker.offset = endMarker.offset = offset;
          startMarker.marker = endMarker.marker = "";
          return;
        }
        startMarker.container.data = `${startMarker.container.data.substring(
          0,
          startMarker.offset
        )}${startMarker.container.data.substring(
          startMarker.offset + 1,
          endMarker.offset
        )}${startMarker.container.data.substring(endMarker.offset + 1)}`;
        if (startMarker.offset >= startMarker.container.length) {
          startMarker.offset = endMarker.offset = startMarker.container.length;
          return;
        }
        endMarker.offset--; // remove the start marker's length
        if (endMarker.offset > endMarker.container.length) {
          endMarker.offset = endMarker.container.length;
        }
        return;
      }
      if (startMarker.container.length === 1) {
        let { container, offset } = removeNode(startMarker.container);
        startMarker.container = container;
        startMarker.offset = offset;
        startMarker.marker = "";
      } else {
        startMarker.container.data = `${startMarker.container.data.substring(
          0,
          startMarker.offset
        )}${startMarker.container.data.substring(startMarker.offset + 1)}`;
      }
      if (endMarker.container.length === 1) {
        let { container, offset } = removeNode(endMarker.container);
        endMarker.container = container;
        endMarker.offset = offset;
        endMarker.marker = "";
      } else {
        endMarker.container.data = `${endMarker.container.data.substring(
          0,
          endMarker.offset
        )}${endMarker.container.data.substring(endMarker.offset + 1)}`;
      }
    })();
    (function handleNodeSelectMarker() {
      if (startMarker.marker === "{") {
        if (startMarker.offset === 0) {
          // The range start with the text node.
          let container = startMarker.container.parentElement;
          startMarker.offset = indexOfContainer(
            container,
            startMarker.container
          );
          startMarker.container = container;
        } else if (startMarker.offset === startMarker.container.data.length) {
          // The range start after the text node.
          let container = startMarker.container.parentElement;
          startMarker.offset =
            indexOfContainer(container, startMarker.container) + 1;
          startMarker.container = container;
        } else {
          throw 'Start marker "{" is allowed start or end of a text node';
        }
      }
      if (endMarker.marker === "}") {
        if (endMarker.offset === 0) {
          // The range ends before the text node.
          let container = endMarker.container.parentElement;
          endMarker.offset = indexOfContainer(container, endMarker.container);
          endMarker.container = container;
        } else if (endMarker.offset === endMarker.container.data.length) {
          // The range ends with the text node.
          let container = endMarker.container.parentElement;
          endMarker.offset =
            indexOfContainer(container, endMarker.container) + 1;
          endMarker.container = container;
        } else {
          throw 'End marker "}" is allowed start or end of a text node';
        }
      }
    })();
    let range = document.createRange();
    range.setStart(startMarker.container, startMarker.offset);
    range.setEnd(endMarker.container, endMarker.offset);
    return range;
  }

  let ranges = [];
  for (
    let range = getNextRangeAndDeleteMarker(gEditor.firstChild);
    range;
    range = getNextRangeAndDeleteMarker(range.endContainer)
  ) {
    ranges.push(range);
  }

  gSelection.removeAllRanges();
  for (let range of ranges) {
    gSelection.addRange(range);
  }

  if (gSelection.rangeCount != ranges.length) {
    throw `Failed to set selection to the given ranges whose length is ${ranges.length}, but only ${gSelection.rangeCount} ranges are added`;
  }
}
