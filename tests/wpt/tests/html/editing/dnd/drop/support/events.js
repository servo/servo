setup({ explicit_timeout: true, single_test: true });
function rAF() {
  return new Promise(resolve => {
    requestAnimationFrame(resolve);
  });
}
const a = document.getElementById('a');
const b = document.getElementById('b');
const actualEvents = [];
const expectedEvents = document.body.dataset.expectedEvents.replace(/\s+/g, '').split(',');
const eventTypes = new Set(expectedEvents.map(s => s.split(':')[1]));
for (const eventType of eventTypes) {
  if (a) {
    a.addEventListener(eventType, e => {
      actualEvents.push(`a:${e.type}:${e.inputType || ''}`);
    });
  }
  b.addEventListener(eventType, async (e) => {
    actualEvents.push(`b:${e.type}:${e.inputType || ''}`);
    if (e.type === "input") {
      await rAF();
      await rAF();
      assert_array_equals(actualEvents, expectedEvents);
      done();
    }
  });
}
const dragMeElement = document.querySelector('[data-select]');
const [selectionStart, selectionEnd] = dragMeElement.dataset.select.split(',').map(s => parseInt(s, 10));
setSelection(dragMeElement, selectionStart, selectionEnd);
dragMeElement.focus();
