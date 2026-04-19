// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function basic() {
  assert.sameValue("a".at(0), "a");
  assert.sameValue("a".at(-1), "a");

  assert.sameValue("".at(0), undefined);
  assert.sameValue("".at(-1), undefined);
  assert.sameValue("".at(1), undefined);

  assert.sameValue("ab".at(0), "a");
  assert.sameValue("ab".at(1), "b");
  assert.sameValue("ab".at(-2), "a");
  assert.sameValue("ab".at(-1), "b");

  assert.sameValue("ab".at(2), undefined);
  assert.sameValue("ab".at(-3), undefined);
  assert.sameValue("ab".at(-4), undefined);
  assert.sameValue("ab".at(Infinity), undefined);
  assert.sameValue("ab".at(-Infinity), undefined);
  assert.sameValue("ab".at(NaN), "a"); // ToInteger(NaN) = 0

  assert.sameValue("\u{1f921}".at(0), "\u{d83e}");
  assert.sameValue("\u{1f921}".at(1), "\u{dd21}");
}

function other() {
  var n = 146;
  assert.sameValue(String.prototype.at.call(n, 0), "1");
  var obj = {};
  assert.sameValue(String.prototype.at.call(obj, -1), "]");
  var b = true;
  assert.sameValue(String.prototype.at.call(b, 0), "t");
}

basic();
other();

