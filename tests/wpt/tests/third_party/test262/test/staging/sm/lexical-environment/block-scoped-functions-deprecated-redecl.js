// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

{
  assert.sameValue(f(), 4);
  function f() { return 3; }
  assert.sameValue(f(), 4);
  function f() { return 4; }
  assert.sameValue(f(), 4);
}

// Annex B still works.
assert.sameValue(f(), 4);

// The same thing with labels.
{
  assert.sameValue(f(), 4);
  function f() { return 3; }
  assert.sameValue(f(), 4);
  l: function f() { return 4; }
  assert.sameValue(f(), 4);
}

// Annex B still works.
assert.sameValue(f(), 4);

function test() {
  {
    assert.sameValue(f(), 2);
    function f() { return 1; }
    assert.sameValue(f(), 2);
    function f() { return 2; }
    assert.sameValue(f(), 2);
  }

  // Annex B still works.
  assert.sameValue(f(), 2);
}

test();

// Strict mode still cannot redeclare.
assert.throws(SyntaxError, function() {
  eval(`"use strict";
  {
    function f() { }
    function f() { }
  }`);
});

// Redeclaring an explicitly 'let'-declared binding doesn't work.
assert.throws(SyntaxError, function() {
  eval(`{
    let x = 42;
    function x() {}
  }`);
});

// Redeclaring an explicitly 'const'-declared binding doesn't work.
assert.throws(SyntaxError, function() {
  eval(`{
    const x = 42;
    function x() {}
  }`);
});
