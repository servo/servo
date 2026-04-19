// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: String.prototype.indexOf type coercion for searchString parameter
esid: sec-string.prototype.indexof
info: |
  String.prototype.indexOf ( searchString [ , position ] )

  3. Let searchStr be ? ToString(searchString).
---*/

assert.sameValue("foo".indexOf(""), 0);
assert.sameValue("__foo__".indexOf("foo"), 2);
assert.sameValue("__undefined__".indexOf(undefined), 2, 'ToString: undefined => "undefined"');
assert.sameValue("__null__".indexOf(null), 2, 'ToString: null => "null"');
assert.sameValue("__true__".indexOf(true), 2, 'ToString: true => "true"');
assert.sameValue("__false__".indexOf(false), 2, 'ToString: false => "false"');
assert.sameValue("__0__".indexOf(0), 2, "ToString: Number to String");
assert.sameValue("__0__".indexOf(-0), 2, 'ToString: -0 => "0"');
assert.sameValue("__Infinity__".indexOf(Infinity), 2, 'ToString: Infinity => "Infinity"');
assert.sameValue("__-Infinity__".indexOf(-Infinity), 2, 'ToString: -Infinity => "-Infinity"');
assert.sameValue("__NaN__".indexOf(NaN), 2, 'ToString: NaN => "NaN"');
assert.sameValue("__123.456__".indexOf(123.456), 2, "ToString: Number to String");
assert.sameValue("__-123.456__".indexOf(-123.456), 2, "ToString: Number to String");
assert.sameValue("foo".indexOf([]), 0, "ToString: .toString()");
assert.sameValue("__foo,bar__".indexOf(["foo", "bar"]), 2, "ToString: .toString()");
assert.sameValue("__[object Object]__".indexOf({}), 2, "ToString: .toString()");
