// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: String.prototype.indexOf type coercion for position parameter
esid: sec-string.prototype.indexof
info: |
  String.prototype.indexOf ( searchString [ , position ] )

  4. Let pos be ? ToInteger(position).
---*/

assert.sameValue("aaaa".indexOf("aa", 0), 0);
assert.sameValue("aaaa".indexOf("aa", 1), 1);
assert.sameValue("aaaa".indexOf("aa", -0.9), 0, "ToInteger: truncate towards 0");
assert.sameValue("aaaa".indexOf("aa", 0.9), 0, "ToInteger: truncate towards 0");
assert.sameValue("aaaa".indexOf("aa", 1.9), 1, "ToInteger: truncate towards 0");
assert.sameValue("aaaa".indexOf("aa", NaN), 0, "ToInteger: NaN => 0");
assert.sameValue("aaaa".indexOf("aa", Infinity), -1);
assert.sameValue("aaaa".indexOf("aa", undefined), 0, "ToInteger: undefined => NaN => 0");
assert.sameValue("aaaa".indexOf("aa", null), 0, "ToInteger: null => 0");
assert.sameValue("aaaa".indexOf("aa", false), 0, "ToInteger: false => 0");
assert.sameValue("aaaa".indexOf("aa", true), 1, "ToInteger: true => 1");
assert.sameValue("aaaa".indexOf("aa", "0"), 0, "ToInteger: parse Number");
assert.sameValue("aaaa".indexOf("aa", "1.9"), 1, "ToInteger: parse Number => 1.9 => 1");
assert.sameValue("aaaa".indexOf("aa", "Infinity"), -1, "ToInteger: parse Number");
assert.sameValue("aaaa".indexOf("aa", ""), 0, "ToInteger: unparseable string => NaN => 0");
assert.sameValue("aaaa".indexOf("aa", "foo"), 0, "ToInteger: unparseable string => NaN => 0");
assert.sameValue("aaaa".indexOf("aa", "true"), 0, "ToInteger: unparseable string => NaN => 0");
assert.sameValue("aaaa".indexOf("aa", 2), 2);
assert.sameValue("aaaa".indexOf("aa", "2"), 2, "ToInteger: parse Number");
assert.sameValue("aaaa".indexOf("aa", 2.9), 2, "ToInteger: truncate towards 0");
assert.sameValue("aaaa".indexOf("aa", "2.9"), 2, "ToInteger: parse Number => truncate towards 0");
assert.sameValue("aaaa".indexOf("aa", [0]), 0, 'ToInteger: [0].toString() => "0" => 0');
assert.sameValue("aaaa".indexOf("aa", ["1"]), 1, 'ToInteger: ["1"].toString() => "1" => 1');
assert.sameValue("aaaa".indexOf("aa", {}), 0,
  'ToInteger: ({}).toString() => "[object Object]" => NaN => 0');
assert.sameValue("aaaa".indexOf("aa", []), 0, 'ToInteger: [].toString() => "" => NaN => 0');
