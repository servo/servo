'use strict';

const DropPosition = Object.freeze({
  CENTER: 'center',
  RIGHT_SCROLLBAR: 'right_scrollbar',
  LEFT_SCROLLBAR: 'left_scrollbar',
  HORIZONTAL_SCROLLBAR: 'horizontal_scrollbar',
});

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

// This is a helper method that moves the pointer to the specified
// position (center or scrollbar) of the element.
const movePointerToPosition = (element, iframe, position, actions) => {
  if (position === DropPosition.CENTER) {
    return movePointerToCenter(element, iframe, actions);
  } else {
    return movePointerToScrollbar(element, iframe, position, actions);
  }
}

// This method appends a pointer move action to the `actions` argument that
// moves the pointer to the center of the `element` and returns it.
const movePointerToCenter = (element, iframe, actions) => {
  return (iframe == undefined) ? actions.pointerMove(0, 0, {
    origin: element
  }) : actions.pointerMove(...getElemCenterInIframe(element, iframe))
}

// Moves the pointer to the center of the specified scrollbar of the element.
const movePointerToScrollbar = (element, iframe, scrollbarPosition, actions) => {

  const thickness = calculateScrollbarThickness();
  assert_greater_than(thickness, 0,
    'movePointerToScrollbar should not be called when overlay scrollbars are enabled');

  const hasVerticalScrollbar = (element, iframe) => {
    if (iframe == undefined) {
      return element.scrollHeight > element.clientHeight;
    }
    // If the element is in an iframe, it will become scrollable if
    // its scrollHeight is larger than the containing frame.
    return element.scrollHeight > iframe.clientHeight;
  };

  const hasHorizontalScrollbar = (element, iframe) => {
  if (iframe == undefined) {
      return element.scrollWidth > element.clientWidth;
    }
    // If the element is in an iframe, it will become scrollable if
    // its scrollWidth is larger than the containing frame.
    return element.scrollWidth > iframe.clientWidth;
  };

  // If the element is inside a frame, the tests will attempt to drop over the document's root
  // scrollbars. With this in mind, we calculate the scrollbar's position relative to the frame's
  // rectangle instead of the inner element.
  const rect = iframe ? iframe.getBoundingClientRect() : element.getBoundingClientRect();
  let x, y;

  if (scrollbarPosition === DropPosition.LEFT_SCROLLBAR &&
      hasVerticalScrollbar(element, iframe)) {
    x = rect.left + thickness / 2;
    y = rect.top + (rect.height / 2);
  } else if (scrollbarPosition === DropPosition.RIGHT_SCROLLBAR &&
      hasVerticalScrollbar(element, iframe)) {
    x = rect.right - thickness / 2;
    y = rect.top + (rect.height / 2);
  } else if (scrollbarPosition === DropPosition.HORIZONTAL_SCROLLBAR &&
      hasHorizontalScrollbar(element, iframe)) {
    // Horizontal scrollbar is positioned at the bottom.
    x = rect.left + (rect.width / 2);
    y = rect.bottom - thickness / 2;
  } else {
    throw new Error('Invalid position specified for scrollbar.');
  }

  return actions.pointerMove(x, y);
}

// The dragDropTest function can be used for tests which require the drag and drop movement.
// `dragElement` takes the element that needs to be dragged. `dropElement` and `dropPosition`
// is where you want to drop the `dragElement` on. By default, `dropPosition` is CENTER,
// which means the center of the `dropElement`. And it can also target to the scrollbar of
// `dropElement` (see DropPosition enum). `onDropCallBack` is called on the onDrop handler
// and the test will only pass if this function returns true. Also, if the `dropElement`
// is inside an iframe, use the optional `iframe` parameter to specify an iframe element
// that contains the `dropElement` to ensure that tests with an iframe pass.
function dragDropTest(dragElement, dropElement, onDropCallBack, testDescription,
  dragIframe = undefined, dropIframe = undefined, dropPosition = DropPosition.CENTER) {
  // Only verifies drop on scrollbar tests if non-overlay scrollbar is present.
  // Skips the test on platforms with overlay scrollbars.
  if (dropPosition !== DropPosition.CENTER && calculateScrollbarThickness() <= 0) {
    return;
  }
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
      actions = movePointerToPosition(dropElement, dropIframe, dropPosition, actions)
        .pointerUp();
      await actions.send();
    } catch (e) {
      reject(e);
    }
  }, testDescription));
}

// The dragDropTestNoDropEvent function performs a drag-and-drop test but expects
// no drop event to occur. This is useful for testing scenarios where drag-and-drop
// should be blocked or ignored (e.g., dropping on root scrollbars). The test
// passes if no drop event fires within the timeout period, and fails immediately
// if any drop event occurs.
function dragDropTestNoDropEvent(dragElement, dropElement, testDescription,
  dragIframe = undefined, dropIframe = undefined, dropPosition = DropPosition.CENTER) {
  // Only verifies drop on scrollbar tests if non-overlay scrollbar is present.
  // Skips the test on platforms with overlay scrollbars.
  if (dropPosition !== DropPosition.CENTER && calculateScrollbarThickness() <= 0) {
    return;
  }
  promise_test((t) => new Promise(async (resolve, reject) => {
    let dropEvent = false;

    dropElement.addEventListener('drop', t.step_func((event) => {
      dropEvent = true;
      reject(new Error('Drop event should not have fired'));
    }));

    try {
      var actions = new test_driver.Actions();
      actions = movePointerToCenter(dragElement, dragIframe, actions)
        .pointerDown();
      actions = movePointerToPosition(dropElement, dropIframe, dropPosition, actions)
        .pointerUp();
      await actions.send();

      if (!dropEvent) {
        resolve();
      }
    } catch (e) {
      reject(e);
    }
  }, testDescription));
}

const calculateScrollbarThickness = () => {
    var container = document.createElement("div");
    container.style.width = "100px";
    container.style.height = "100px";
    container.style.position = "absolute";
    container.style.visibility = "hidden";
    container.style.overflow = "auto";

    document.body.appendChild(container);

    var widthBefore = container.clientWidth;
    var longContent = document.createElement("div");
    longContent.style.height = "1000px";
    container.appendChild(longContent);

    var widthAfter = container.clientWidth;

    container.remove();

    return widthBefore - widthAfter;
}
