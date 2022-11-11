function waitForRender() {
  return new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
}
async function clickOn(element) {
  const actions = new test_driver.Actions();
  await waitForRender();
  await actions.pointerMove(0, 0, {origin: element})
      .pointerDown({button: actions.ButtonType.LEFT})
      .pointerUp({button: actions.ButtonType.LEFT})
      .send();
  await waitForRender();
}
async function sendTab() {
  await waitForRender();
  const kTab = '\uE004';
  await new test_driver.send_keys(document.body,kTab);
  await waitForRender();
}
// Waiting for crbug.com/893480:
// async function sendShiftTab() {
//   await waitForRender();
//   const kShift = '\uE008';
//   const kTab = '\uE004';
//   await new test_driver.Actions()
//     .keyDown(kShift)
//     .keyDown(kTab)
//     .keyUp(kTab)
//     .keyUp(kShift)
//     .send();
//   await waitForRender();
// }
async function sendEscape() {
  await waitForRender();
  await new test_driver.send_keys(document.body,'\uE00C'); // Escape
  await waitForRender();
}
async function sendEnter() {
  await waitForRender();
  await new test_driver.send_keys(document.body,'\uE007'); // Enter
  await waitForRender();
}
function isElementVisible(el) {
  return !!(el.offsetWidth || el.offsetHeight || el.getClientRects().length);
}
async function finishAnimations(popover) {
  popover.getAnimations({subtree: true}).forEach(animation => animation.finish());
  await waitForRender();
}
let mouseOverStarted;
function mouseOver(element) {
  mouseOverStarted = performance.now();
  return (new test_driver.Actions())
    .pointerMove(0, 0, {origin: element})
    .send();
}
function msSinceMouseOver() {
  return performance.now() - mouseOverStarted;
}
async function waitForHoverTime(hoverWaitTimeMs) {
  await new Promise(resolve => step_timeout(resolve,hoverWaitTimeMs));
  await waitForRender();
};
