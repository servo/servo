"use strict";

test(() => {
  assert_throws("DataCloneError", () => {
    new Notification("Bob: Hi", { data: new SharedArrayBuffer() });
  })
}, "SharedArrayBuffer cloning via the Notifications API's data member: basic case");

test(() => {
  let getter1Called = false;
  let getter2Called = false;

  assert_throws("DataCloneError", () => {
    new Notification("Bob: Hi", { data: [
      { get x() { getter1Called = true; return 5; } },
      new SharedArrayBuffer(),
      { get x() { getter2Called = true; return 5; } }
    ]});
  });

  assert_true(getter1Called, "The getter before the SAB must have been called");
  assert_false(getter2Called, "The getter after the SAB must not have been called");
}, "SharedArrayBuffer cloning via the Notifications API's data member: is interleaved correctly");
