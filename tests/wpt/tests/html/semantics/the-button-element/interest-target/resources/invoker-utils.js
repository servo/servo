function waitForRender() {
  return new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
}
async function clickOn(element) {
  await waitForRender();
  let rect = element.getBoundingClientRect();
  let actions = new test_driver.Actions();
  // FIXME: Switch to pointerMove(0, 0, {origin: element}) once
  // https://github.com/web-platform-tests/wpt/issues/41257 is fixed.
  await actions
      .pointerMove(Math.round(rect.x + rect.width / 2), Math.round(rect.y + rect.height / 2), {})
      .pointerDown({button: actions.ButtonType.LEFT})
      .pointerUp({button: actions.ButtonType.LEFT})
      .send();
  await waitForRender();
}
async function hoverOver(element) {
  await waitForRender();
  let rect = element.getBoundingClientRect();
  let actions = new test_driver.Actions();
  // FIXME: Switch to pointerMove(0, 0, {origin: element}) once
  // https://github.com/web-platform-tests/wpt/issues/41257 is fixed.
  await actions
      .pointerMove(Math.round(rect.x + rect.width / 2), Math.round(rect.y + rect.height / 2), {})
      .send();
  await waitForRender();
}
let mousemoveInfo;
function mouseOverAndRecord(element) {
  mousemoveInfo?.controller.abort();
  const controller = new AbortController();
  mousemoveInfo = {element, controller, moved: false, started: performance.now()};
  document.addEventListener("mousemove", (e) => {mousemoveInfo.moved = true;}, {signal: controller.signal});
  return (new test_driver.Actions())
    .pointerMove(0, 0, {origin: element})
    .send();
}
// Note that this may err on the side of being too large (reporting a number
// that is larger than the actual time since the mouseover happened), due to how
// `mousemoveInfo.started` is initialized, on first mouse move. However, this
// function is intended to be used as a detector for the test harness taking too
// long for some tests, so it's ok to be conservative.
function msSinceMouseOver() {
  return performance.now() - mousemoveInfo.started;
}
function assertMouseStillOver(element) {
  assert_equals(mousemoveInfo.element, element, 'Broken test harness');
  assert_false(mousemoveInfo.moved,'Broken test harness');
}
async function waitForHoverTime(hoverWaitTimeMs) {
  await new Promise(resolve => step_timeout(resolve,hoverWaitTimeMs));
  await waitForRender();
};

function createPopoverAndInvokerForHoverTests(test, showdelayMs, hideDelayMs) {
  const popover = document.createElement('div');
  popover.popover = 'auto';
  popover.setAttribute('style','top: 200px;');
  popover.textContent = 'Popover';
  document.body.appendChild(popover);
  let invoker = document.createElement('button');
  invoker.interestTargetElement = popover;
  invoker.setAttribute('style',`
    interest-target-show-delay: ${showdelayMs}ms;
    interest-target-hide-delay: ${hideDelayMs}ms;
    position:relative;
    top:100px;
    width:fit-content;
    height:fit-content;
    `);
  invoker.innerText = 'Invoker';
  document.body.appendChild(invoker);
  const actualShowDelay = Number(getComputedStyle(invoker).interestTargetShowDelay.slice(0,-1))*1000;
  assert_equals(actualShowDelay,showdelayMs,'interest-target-show-delay is incorrect');
  const actualHideDelay = Number(getComputedStyle(invoker).interestTargetHideDelay.slice(0,-1))*1000;
  assert_equals(actualHideDelay,hideDelayMs,'interest-target-hide-delay is incorrect');
  const unrelated = document.createElement('div');
  document.body.appendChild(unrelated);
  unrelated.textContent = 'Unrelated';
  unrelated.setAttribute('style','position:relative; top:0;');
  test.add_cleanup(async () => {
    popover.remove();
    invoker.remove();
    unrelated.remove();
    await waitForRender();
  });
  return {popover, invoker, unrelated};
}
