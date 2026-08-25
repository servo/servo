/*---
description: An async Test262 smoketest that throws an unexpected error
flags: [async]
---*/

setTimeout(function() {
  foo.bar(); // Throws unexpected ReferenceError
}, 10);
