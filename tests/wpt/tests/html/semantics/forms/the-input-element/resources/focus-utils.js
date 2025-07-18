'use strict';

function waitForRender() {
  return new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
}

async function navigateFocusForward() {
  await waitForRender();
  const kTab = '\uE004';
  await new test_driver.Actions()
      .keyDown(kTab)
      .keyUp(kTab)
      .send();
  await waitForRender();
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
