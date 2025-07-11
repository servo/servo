'use strict';

// This method calculates the center of an element in an iframe in the
// coordinate space of the top frame. We need this because TestDriver doesn't
// support Actions `{origin}`s across two different frames.
const getElemCenterInIframe = (element, iframe) => {
  const elemClientRect = element.getBoundingClientRect();
  const frameClientRect = iframe.getBoundingClientRect();
  const centerX = frameClientRect.left + (elemClientRect.left + elemClientRect
    .right) / 2;
  const centerY = frameClientRect.top + (elemClientRect.top + elemClientRect
    .bottom) / 2;
  return [centerX, centerY];
};

// This method appends a pointer move action to the `actions` argument that
// moves the pointer to the center of the `element` and returns it.
const movePointerToCenter = (element, iframe, actions) => {
  return (iframe == undefined) ? actions.pointerMove(0, 0, {
    origin: element
  }) : actions.pointerMove(...getElemCenterInIframe(element, iframe))
}

// The dragDropTest function can be used for tests which require the drag and drop movement.
// `dragElement` takes the element that needs to be dragged and `dropElement` is the element which
// you want to drop the `dragElement` on. `onDropCallback` is called on the onDrop handler and the
// test will only pass if this function returns true. Also, if the `dropElement` is inside an
// iframe, use the optional `iframe` parameter to specify an iframe element that contains the
// `dropElement` to ensure that tests with an iframe pass.
// TODO(https://crbug.com/426228061): Some tests were written to drag into scrollbars
// instead of the center of the element, this function should be expanded to accommodate them.
function dragDropTest(dragElement, dropElement, onDropCallBack, testDescription,
  dragIframe = undefined, dropIframe = undefined) {
  promise_test((t) => new Promise(async (resolve, reject) => {
    dropElement.addEventListener('drop', t.step_func((event) => {
      if (onDropCallBack(event) == true) {
        resolve();
      } else {
        reject();
      }
    }));
    try {
      var actions = new test_driver.Actions();
      actions = movePointerToCenter(dragElement, dragIframe, actions)
        .pointerDown();
      actions = movePointerToCenter(dropElement, dropIframe, actions)
        .pointerUp();
      await actions.send();
    } catch (e) {
      reject(e);
    }
  }, testDescription));
}
