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
function mouseOverAndRecord(t,element) {
  let mouseMoveInfo;
  t.add_cleanup(() => mouseMoveInfo?.controller.abort());
  const controller = new AbortController();
  mouseMoveInfo = {element, controller, moved: false, started: performance.now()};
  document.addEventListener("mousemove", (e) => {mouseMoveInfo.moved = true;}, {signal: controller.signal});
  return (new test_driver.Actions())
      .pointerMove(0, 0, {origin: element})
      .send()
      .then(() => mouseMoveInfo);
}
// Note that this may err on the side of being too large (reporting a number
// that is larger than the actual time since the mouseover happened), due to how
// `mousemoveInfo.started` is initialized, on first mouse move. However, this
// function is intended to be used as a detector for the test harness taking too
// long for some tests, so it's ok to be conservative.
function msSinceMouseOver(mouseMoveInfo) {
  return performance.now() - mouseMoveInfo.started;
}
async function waitForHoverTime(hoverWaitTimeMs) {
  await new Promise(resolve => step_timeout(resolve,hoverWaitTimeMs));
  await waitForRender();
};

async function createPopoverAndInvokerForHoverTests(test, showdelayMs, hideDelayMs) {
  const unrelated = document.createElement('div');
  document.body.appendChild(unrelated);
  unrelated.textContent = 'Unrelated';
  unrelated.setAttribute('style','position:fixed; top:0;');
  // Ensure we never hover over an active interesttarget element.
  await hoverOver(unrelated);
  const popover = document.createElement('div');
  popover.popover = 'auto';
  popover.setAttribute('style','inset:auto; top: 100px;');
  popover.textContent = 'Popover';
  document.body.appendChild(popover);
  let invoker = document.createElement('button');
  invoker.interestTargetElement = popover;
  invoker.setAttribute('style',`
    interest-target-show-delay: ${showdelayMs}ms;
    interest-target-hide-delay: ${hideDelayMs}ms;
    position:fixed;
    top:200px;
    width:fit-content;
    height:fit-content;
    `);
  invoker.innerText = 'Invoker';
  document.body.appendChild(invoker);
  const actualShowDelay = Number(getComputedStyle(invoker).interestTargetShowDelay.slice(0,-1))*1000;
  assert_equals(actualShowDelay,showdelayMs,'interest-target-show-delay is incorrect');
  const actualHideDelay = Number(getComputedStyle(invoker).interestTargetHideDelay.slice(0,-1))*1000;
  assert_equals(actualHideDelay,hideDelayMs,'interest-target-hide-delay is incorrect');
  test.add_cleanup(async () => {
    popover.remove();
    invoker.remove();
    unrelated.remove();
    await waitForRender();
  });
  assert_false(popover.matches(':popover-open'),'The popover should start out closed');
  return {popover, invoker, unrelated};
}
