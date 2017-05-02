"use strict";

test(() => {
  assert_equals(console.timeline, undefined, "console.timeline should be undefined");
}, "'timeline' function should not exist on the console object");

test(() => {
  assert_equals(console.timelineEnd, undefined, "console.timelineEnd should be undefined");
}, "'timelineEnd' function should not exist on the console object");