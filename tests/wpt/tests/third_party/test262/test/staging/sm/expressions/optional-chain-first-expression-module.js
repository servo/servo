// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
flags:
  - module
---*/
// Verify bytecode emitter accepts all valid optional chain first expressions.

// Evaluate `import.meta` in an expression context.
assert.throws(TypeError, function() {
  void (import.meta?.());
});
assert.throws(TypeError, function() {
  void (import.meta?.p());
});

// Also try `import.meta` parenthesized.
assert.throws(TypeError, function() {
  void ((import.meta)?.());
});
assert.throws(TypeError, function() {
  void ((import.meta)?.p());
});

// Evaluate `import()` in an expression context.
assert.throws(TypeError, function() {
  void (import("./optional-chain-first-expression-module_FIXTURE.js")?.());
});
assert.throws(TypeError, function() {
  void (import("./optional-chain-first-expression-module_FIXTURE.js")?.p());
});

// Also try `import()` parenthesized.
assert.throws(TypeError, function() {
  void ((import("./optional-chain-first-expression-module_FIXTURE.js"))?.());
});
assert.throws(TypeError, function() {
  void ((import("./optional-chain-first-expression-module_FIXTURE.js"))?.p());
});
