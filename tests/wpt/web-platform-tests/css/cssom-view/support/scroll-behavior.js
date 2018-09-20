function observeScrolling(elements, callback) {
  if (!Array.isArray(elements))
      elements = [elements];
  var lastChangedFrame = 0;
  var lastLeft = new Map();
  var lastTop = new Map();
  elements.forEach((element) => {
    lastLeft.set(element, element.scrollLeft);
    lastTop.set(element, element.scrollTop);
  });
  function tick(frames) {
    // We requestAnimationFrame either for 500 frames or until 20 frames with
    // no change have been observed.
    if (frames >= 500 || frames - lastChangedFrame > 20) {
      callback(true);
    } else {
      var scrollHappened = elements.some((element) => {
        return element.scrollLeft != lastLeft.get(element) || element.scrollTop != lastTop.get(element);
      });
      if (scrollHappened) {
        lastChangedFrame = frames;
        elements.forEach((element) => {
          lastLeft.set(element, element.scrollLeft);
          lastTop.set(element, element.scrollTop);
        });
        callback(false);
      }
      requestAnimationFrame(tick.bind(null, frames + 1));
    }
  }
  tick(0);
}

function waitForScrollEnd(elements) {
  return new Promise((resolve) => {
    observeScrolling(elements, (done) => {
      if (done)
        resolve();
    });
  });
}

function resetScroll(scrollingElement) {
  // Try various methods to ensure the element position is reset immediately.
  scrollingElement.scrollLeft = 0;
  scrollingElement.scrollTop = 0;
  scrollingElement.scroll({left: 0, top: 0, behavior: "instant"});
}

function resetScrollForWindow(scrollingWindow) {
  // Try various methods to ensure the element position is reset immediately.
  scrollingWindow.document.scrollingElement.scrollLeft = 0;
  scrollingWindow.document.scrollingElement.scrollTop = 0;
  scrollingWindow.scroll({left: 0, top: 0, behavior: "instant"});
}

function setScrollBehavior(styledElement, className) {
  styledElement.classList.remove("autoBehavior", "smoothBehavior");
  styledElement.classList.add(className);
}

function scrollNode(scrollingElement, scrollFunction, behavior, elementToRevealLeft, elementToRevealTop) {
  var args = {};
  if (behavior)
    args.behavior = behavior;
  switch (scrollFunction) {
    case "scrollIntoView":
      args.inline = "start";
      args.block = "start";
      elementToReveal.scrollIntoView(args);
      break;
    default:
      args.left = elementToRevealLeft;
      args.top = elementToRevealTop;
      scrollingElement[scrollFunction](args);
      break;
  }
}

function scrollWindow(scrollingWindow, scrollFunction, behavior, elementToRevealLeft, elementToRevealTop) {
  var args = {};
  if (behavior)
    args.behavior = behavior;
  args.left = elementToRevealLeft;
  args.top = elementToRevealTop;
  scrollingWindow[scrollFunction](args);
}
