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
  await new test_driver.send_keys(document.body,'\uE004'); // Tab
  await waitForRender();
}
async function sendEscape() {
  await waitForRender();
  await new test_driver.send_keys(document.body,'\uE00C'); // Escape
  await waitForRender();
}
