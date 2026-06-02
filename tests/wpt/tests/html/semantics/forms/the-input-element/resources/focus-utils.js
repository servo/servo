'use strict';

function waitForRender() {
  return new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
}

async function pressKey(keyCode) {
  await waitForRender();
  await new test_driver.Actions()
    .keyDown(keyCode)
    .keyUp(keyCode)
    .send();
  await waitForRender();
}

async function arrowUp() {
  const kArrowUp = '\uE013';
  await pressKey(kArrowUp);
}

async function arrowDown() {
  const kArrowDown = '\uE015';
  await pressKey(kArrowDown);
}

async function navigateFocusForward() {
  const kTab = '\uE004';
  await pressKey(kTab);
}

async function navigateFocusBackward() {
  await waitForRender();
  const kShift = '\uE008';
  const kTab = '\uE004';
  await new test_driver.Actions()
    .keyDown(kShift)
    .keyDown(kTab)
    .keyUp(kTab)
    .keyUp(kShift)
    .send();
  await waitForRender();
}
