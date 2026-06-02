'use strict';

const controller = new CaptureController();
const type = 'my-event-type';
const listeners = {};
const listener_count = 10;
for (let i = 0; i < listener_count; i++) {
  listeners[i] = {
    callback: (event) => {
      assert_equals(event.type, type, `Event type sent to listener ${i}`);
      listeners[i].execution_count++;
    }
  };
}

test(() => {
  for (const i in listeners) {
    listeners[i].execution_count = 0;
    controller.addEventListener(type, listeners[i].callback);
  }
  controller.dispatchEvent(new Event(type));
  for (const i in listeners) {
    assert_equals(
        listeners[i].execution_count, 1,
        `Callback execution count for listener ${i}`);
  }
}, 'Registering listeners on CaptureController and dispatching an event.');

test(() => {
  for (const i in listeners) {
    listeners[i].execution_count = 0;
  }
  controller.dispatchEvent(new Event(type));
  controller.dispatchEvent(new Event(type));
  controller.dispatchEvent(new Event(type));
  for (const i in listeners) {
    assert_equals(
        listeners[i].execution_count, 3,
        `Callback execution count for listener ${i}`);
  }
}, 'Dispatching an multiple events to CaptureController.');

test(() => {
  for (const i in listeners) {
    listeners[i].execution_count = 0;
    if (i % 3) {
      listeners[i].removed = false;
    } else {
      listeners[i].removed = true;
      controller.removeEventListener(type, listeners[i].callback);
    };
  }
  controller.dispatchEvent(new Event(type));
  controller.dispatchEvent(new Event(type));
  for (const i in listeners) {
    assert_equals(
        listeners[i].execution_count, listeners[i].removed ? 0 : 2,
        `Callback execution count for listener ${i}`);
  }
}, 'Unregistering listeners from CaptureController and dispatching an event.');
