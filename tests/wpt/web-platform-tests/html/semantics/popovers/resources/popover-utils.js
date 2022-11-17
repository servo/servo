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
async function blessTopLayer(visibleElement) {
  // The normal "bless" function doesn't work well when there are top layer
  // elements blocking clicks. Additionally, since the normal test_driver.bless
  // function just adds a button to the main document and clicks it, we can't
  // call that in the presence of open popovers, since that click will close them.
  const button = document.createElement('button');
  button.innerHTML = "Click me to activate";
  visibleElement.appendChild(button);
  let wait_click = new Promise(resolve => button.addEventListener("click", resolve, {once: true}));
  await test_driver.click(button);
  await wait_click;
  button.remove();
}
// This is a "polyfill" of sorts for the `defaultopen` attribute.
// It can be called before window.load is complete, and it will
// show defaultopen popovers according to the rules previously part
// of the popover API: any popover=manual popover can be shown this
// way, and only the first popover=auto popover.
function showDefaultopenPopoversOnLoad() {
  function show() {
    const popovers = Array.from(document.querySelectorAll('[popover][defaultopen]'));
    popovers.forEach((p) => {
        // The showPopover calls below aren't guarded by a check on the popover
        // open/closed status. If they throw exceptions, this function was
        // probably called at a bad time. However, a check is made for open
        // <dialog open> elements.
        if (p instanceof HTMLDialogElement && p.hasAttribute('open'))
          return;
        switch (p.popover) {
          case 'auto':
            if (!document.querySelector('[popover]:open'))
              p.showPopover();
            return;
          case 'manual':
            p.showPopover();
            return;
          default:
            assert_unreached(`Unknown popover type ${p.popover}`);
        }
      });
  }
  if (document.readyState === 'complete') {
    show();
  } else {
    window.addEventListener('load',show,{once:true});
  }
}
