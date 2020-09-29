"use strict";

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

let gSelection = getSelection();
let gEditor = document.querySelector("div[contenteditable]");
let gBeforeinput = [];
let gInput = [];
gEditor.addEventListener("beforeinput", e => {
  // NOTE: Blink makes `getTargetRanges()` return empty range after propagation,
  //       but this test wants to check the result during propagation.
  //       Therefore, we need to cache the result, but will assert if
  //       `getTargetRanges()` returns different ranges after checking the
  //       cached ranges.
  e.cachedRanges = e.getTargetRanges();
  gBeforeinput.push(e);
});
gEditor.addEventListener("input", e => {
  e.cachedRanges = e.getTargetRanges();
  gInput.push(e);
});

function initializeTest(aInnerHTML) {
  gEditor.innerHTML = aInnerHTML;
  gEditor.focus();
  gBeforeinput = [];
  gInput = [];
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

function checkGetTargetRangesKeepReturningSameValue(event) {
  // https://github.com/w3c/input-events/issues/114
  assert_equals(
    getArrayOfRangesDescription(event.getTargetRanges()),
    getArrayOfRangesDescription(event.cachedRanges),
    `${event.type}.getTargetRanges() should keep returning the same array of ranges even after its propagation finished`
  );
}

function checkGetTargetRangesOfBeforeinputOnDeleteSomething(expectedRange) {
  assert_equals(
    gBeforeinput.length,
    1,
    "One beforeinput event should be fired if the key operation deletes something"
  );
  assert_true(
    Array.isArray(gBeforeinput[0].cachedRanges),
    "gBeforeinput[0].getTargetRanges() should return an array of StaticRange instances during propagation"
  );
  // Before checking the length of array of ranges, we should check the first
  // range first because the first range data is more important than whether
  // there are additional unexpected ranges.
  if (gBeforeinput[0].cachedRanges.length > 0) {
    assert_equals(
      getRangeDescription(gBeforeinput[0].cachedRanges[0]),
      getRangeDescription(expectedRange),
      `gBeforeinput[0].getTargetRanges() should return expected range (inputType is "${gBeforeinput[0].inputType}")`
    );
    assert_equals(
      gBeforeinput[0].cachedRanges.length,
      1,
      "gBeforeinput[0].getTargetRanges() should return one range within an array"
    );
  }
  assert_equals(
    gBeforeinput[0].cachedRanges.length,
    1,
    "One range should be returned from getTargetRanges() when the key operation deletes something"
  );
  checkGetTargetRangesKeepReturningSameValue(gBeforeinput[0]);
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
    "gInput[0].getTargetRanges() should return an array of StaticRange instances during propagation"
  );
  assert_equals(
    gInput[0].cachedRanges.length,
    0,
    "gInput[0].getTargetRanges() should return empty array during propagation"
  );
  checkGetTargetRangesKeepReturningSameValue(gInput[0]);
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
