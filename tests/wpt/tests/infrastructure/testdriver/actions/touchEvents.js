function eventEquals(e, expected) {
  for (const prop of Object.keys(expected)) {
    switch (prop) {
      case "screenX":
      case "screenY":
      case "clientX":
      case "clientY":
      case "offsetX":
      case "offsetY":
      case "pageX":
      case "pageY":
        assert_true(
          e[prop] >= expected[prop] - 0.5 &&
            e[prop] <= expected[prop] + 0.5,
          `Event ${e.type} pointerId ${e.pointerId} property ${prop}, expected: ${
            expected[prop]
          } ± 0.5, but got: ${e[prop]}`
        );
        break;
      default:
        assert_equals(e[prop], expected[prop], `Event ${e.type} pointerId ${e.pointerId} property ${prop}`);
        break;
    }
  }
}

function addPointerEventListeners(test, target, events) {
  for (const event of ["pointerup", "pointerdown", "pointermove"]) {
    target.addEventListener(event, test.step_func(e => events.push(e)));
  }
}
