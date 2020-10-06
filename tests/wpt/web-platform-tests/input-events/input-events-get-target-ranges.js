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

  gEditor.innerHTML = aInnerHTML;
  gEditor.focus();
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
