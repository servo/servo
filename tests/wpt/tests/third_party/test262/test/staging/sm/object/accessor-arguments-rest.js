// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
assert.throws(SyntaxError, () => eval("({ get x(...a) { } })"));
assert.throws(SyntaxError, () => eval("({ get x(a, ...b) { } })"));
assert.throws(SyntaxError, () => eval("({ get x([a], ...b) { } })"));
assert.throws(SyntaxError, () => eval("({ get x({a}, ...b) { } })"));
assert.throws(SyntaxError, () => eval("({ get x({a: A}, ...b) { } })"));

assert.throws(SyntaxError, () => eval("({ set x(...a) { } })"));
assert.throws(SyntaxError, () => eval("({ set x(a, ...b) { } })"));
assert.throws(SyntaxError, () => eval("({ set x([a], ...b) { } })"));
assert.throws(SyntaxError, () => eval("({ set x({a: A}, ...b) { } })"));

({ get(...a) { } });
({ get(a, ...b) { } });
({ get([a], ...b) { } });
({ get({a}, ...b) { } });
({ get({a: A}, ...b) { } });

({ set(...a) { } });
({ set(a, ...b) { } });
({ set([a], ...b) { } });
({ set({a: A}, ...b) { } });

