// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function basic() {
  assert.sameValue([0].at(0), 0);
  assert.sameValue([0].at(-1), 0);

  assert.sameValue([].at(0), undefined);
  assert.sameValue([].at(-1), undefined);
  assert.sameValue([].at(1), undefined);

  assert.sameValue([0, 1].at(0), 0);
  assert.sameValue([0, 1].at(1), 1);
  assert.sameValue([0, 1].at(-2), 0);
  assert.sameValue([0, 1].at(-1), 1);

  assert.sameValue([0, 1].at(2), undefined);
  assert.sameValue([0, 1].at(-3), undefined);
  assert.sameValue([0, 1].at(-4), undefined);
  assert.sameValue([0, 1].at(Infinity), undefined);
  assert.sameValue([0, 1].at(-Infinity), undefined);
  assert.sameValue([0, 1].at(NaN), 0); // ToInteger(NaN) = 0
}

function obj() {
  var o = {length: 0, [0]: "a", at: Array.prototype.at};

  assert.sameValue(o.at(0), undefined);
  assert.sameValue(o.at(-1), undefined);

  o.length = 1;
  assert.sameValue(o.at(0), "a");
  assert.sameValue(o.at(-1), "a");
  assert.sameValue(o.at(1), undefined);
  assert.sameValue(o.at(-2), undefined);
}

basic();
obj();

