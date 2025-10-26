const effectAllowedList = ["uninitialized", "undefined", "none", "all",
  "copy",
  "move", "link", "copyMove", "copyLink", "linkMove", "dummy"
];
const dropEffectList = [ "none", "copy", "move", "link", "dummy" ];

// Drop callback used for `dropEffect` tests in `dnd/drop/`. This function
// compares the text content of the drop target with the `dropEffect` and
// `effectAllowed` values of the `dataTransfer` object. The only
// `effectAllowed` values that will be compared are "copy", "move", and "link"
// since they have to correspond to the `dropEffect` value of the event.
function dropEffectOnDropCallBack(event) {
  assert_equals(event.target.textContent, event.dataTransfer.dropEffect);
  assert_equals(event.target.textContent, event.dataTransfer.effectAllowed);
  return true;
}

function buildDragAndDropDivs() {
  effectAllowedList.forEach(effectAllowed => {
    document.getElementById('drag-container').innerHTML +=
      `<div id="drag-${effectAllowed}" draggable="true" ondragstart="event.dataTransfer.effectAllowed = '${effectAllowed}'">${effectAllowed}</div>`;
  });
  dropEffectList.forEach(dropEffect => {
    document.getElementById('drop-container').innerHTML +=
      `<div id="drop-${dropEffect}" ondragover="onDragOver(event, '${dropEffect}')">${dropEffect}</div>`;
  });
}

function expectedDropEffectForEffectAllowed(chosenDropEffect,
  chosenEffectAllowed) {
  if (chosenDropEffect == "dummy") {
    switch (chosenEffectAllowed) {
      case "undefined":
      case "copyLink":
      case "copyMove":
      case "uninitialized":
      case "all":
        return "copy";
      case "linkMove":
        return "link";
      case "move":
        return "move";
      default:
        return chosenEffectAllowed;
    }
  }
  return chosenDropEffect;
}

function dropEventShouldBeSent(dropEffect, effectAllowed) {
  dropEffect = expectedDropEffectForEffectAllowed(dropEffect, effectAllowed);
  if (effectAllowed === 'dummy' || effectAllowed === 'undefined') {
    effectAllowed = 'uninitialized';
  }
  if (effectAllowed === 'none' || dropEffect === 'none') {
    return false;
  }
  if (effectAllowed === 'uninitialized' || effectAllowed === 'all') {
    return true;
  }
  // Matches cases like `copyLink` / `link`.
  if (effectAllowed.toLowerCase().includes(dropEffect)) {
    return true;
  }
  return false;
}

function onDropCallBack(event, chosenDropEffect, chosenEffectAllowed) {
  const actualDropEffect = event.dataTransfer.dropEffect;
  const actualEffectAllowed = event.dataTransfer.effectAllowed;
  let expectedEffectAllowed = chosenEffectAllowed;
  if (chosenEffectAllowed === 'dummy' || chosenEffectAllowed ===
    'undefined') {
    expectedEffectAllowed = 'uninitialized';
  }
  assert_equals(actualEffectAllowed, expectedEffectAllowed,
    `chosenDropEffect: ${chosenDropEffect}, chosenEffectAllowed: ${chosenEffectAllowed}; failed effectAllowed check:`
    );
  let expectedDropEffect = expectedDropEffectForEffectAllowed(
    chosenDropEffect, actualEffectAllowed);
  // `dragend` events with invalid dropEffect-effectAllowed combinations have a
  // `none` dropEffect.
  if (!dropEventShouldBeSent(chosenDropEffect, chosenEffectAllowed)) {
    expectedDropEffect = 'none';
  }
  assert_equals(actualDropEffect, expectedDropEffect,
    `chosenDropEffect: ${chosenDropEffect}, chosenEffectAllowed: ${chosenEffectAllowed}; failed dropEffect check:`
    );
  return true;
}

function onDragOver(event, dropEffect) {
  event.dataTransfer.dropEffect = dropEffect;
  event.preventDefault();
}

// This function creates the divs with all the `effectAllowed`s defined in
// `effectAllowedList` and runs a drag and drop test that verifies that
// the correct events are sent (or not) depending on the combination of
// `dropEffect` and `effectAllowed`.
// `effectAllowed`: string with the `effectAllowed` that will be set on the
//               drag target.
// `dropEffect`: string with the `dropEffect` that will be set on the
//               drop target.
function runDropEffectTestOnDragEnd(effectAllowed, dropEffect) {
  buildDragAndDropDivs();
  const dragDiv = document.getElementById("drag-" + effectAllowed);
  const dropDiv = document.getElementById("drop-" + dropEffect);
  dragEndTest(dragDiv, dropDiv, (e) => onDropCallBack(e,
      dropEffect, effectAllowed),
    `${effectAllowed} / ${dropEffect}`);
}

// Like `runDropEffectTestOnDragEnd`, but verifies that the drop event has the
// correct `dropEffect` and `effectAllowed` values on the drop target, instead
// of `dragEnd` on the drag element.
function runDropEffectTestOnDrop(effectAllowed, dropEffect) {
  buildDragAndDropDivs();
  const dragDiv = document.getElementById("drag-" + effectAllowed);
  const dropDiv = document.getElementById("drop-" + dropEffect);
  const shouldReceiveDropEvent = dropEventShouldBeSent(dropEffect,
    effectAllowed);
  if (shouldReceiveDropEvent) {
    dragDropTest(dragDiv, dropDiv, (e) => onDropCallBack(e,
        dropEffect, effectAllowed),
      `${effectAllowed} / ${dropEffect}`);
  } else {
    dragDropTestNoDropEvent(dragDiv, dropDiv,
      `${effectAllowed} / ${dropEffect}`);
  }
}
