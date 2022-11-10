function eventEquals(e, expected) {
  for (const prop of Object.keys(expected)) {
    assert_equals(e[prop], expected[prop], `Event ${e.type} pointerId ${e.pointerId} property ${prop}`);
  }
}

function addPointerEventListeners(test, target, events) {
  for (const event of ["pointerup", "pointerdown", "pointermove"]) {
    target.addEventListener(event, test.step_func(e => events.push(e)));
  }
}
