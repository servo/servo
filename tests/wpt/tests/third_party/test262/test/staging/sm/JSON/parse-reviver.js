// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

function doubler(k, v)
{
  assert.sameValue(typeof k, "string");

  if (typeof v == "number")
    return 2 * v;

  return v;
}

var x = JSON.parse('{"a":5,"b":6}', doubler);
assert.sameValue(x.hasOwnProperty('a'), true);
assert.sameValue(x.hasOwnProperty('b'), true);
assert.sameValue(x.a, 10);
assert.sameValue(x.b, 12);

x = JSON.parse('[3, 4, 5]', doubler);
assert.sameValue(x[0], 6);
assert.sameValue(x[1], 8);
assert.sameValue(x[2], 10);

// make sure reviver isn't called after a failed parse
var called = false;
function dontCallMe(k, v)
{
  called = true;
}

assert.throws(SyntaxError, function() {
  JSON.parse('{{{{{{{}}}}', dontCallMe);
});
assert.sameValue(called, false);
