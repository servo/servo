const LIME = 'rgb(0, 255, 0)';
const GREEN = 'rgb(0, 128, 0)';
const RED = 'rgb(255, 0, 0)';

function assert_background_color(elementOrId, colorString, comment) {
  const element = typeof elementOrId === 'string' ? document.getElementById(elementOrId) : elementOrId;
  assert_equals(getComputedStyle(element).backgroundColor, colorString, comment);
}

function assert_not_background_color(elementOrId, colorString, comment) {
  const element = typeof elementOrId === 'string' ? document.getElementById(elementOrId) : elementOrId;
  assert_not_equals(getComputedStyle(element).backgroundColor, colorString, comment);
}

function assert_color(elementOrId, colorString, comment) {
  const element = typeof elementOrId === 'string' ? document.getElementById(elementOrId) : elementOrId;
  assert_equals(getComputedStyle(element).color, colorString, comment);
}
