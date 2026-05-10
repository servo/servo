// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Test the groups object with matched and unmatched named captures.
esid: sec-regexpbuiltinexec
features: [regexp-named-groups]
info: |
  Runtime Semantics: RegExpBuiltinExec ( R, S )
    24. If _R_ contains any |GroupName|, then
      a. Let _groups_ be ObjectCreate(*null*).
    25. Else,
      a. Let _groups_ be *undefined*.
    26. Perform ! CreateDataProperty(_A_, `"groups"`, _groups_).
---*/

const re = /(?<a>a).|(?<x>x)/;
const result = re.exec("ab");
assert.sameValue(Object.getPrototypeOf(result), Array.prototype);
assert(result.hasOwnProperty("groups"));
assert.sameValue("ab", result[0]);
assert.sameValue("a", result[1]);
assert.sameValue(undefined, result[2]);
assert.sameValue(0, result.index);
assert.sameValue("a", result.groups.a);
assert.sameValue(undefined, result.groups.x);

// `a` is a matched named capture, `b` is an unmatched named capture, and `z`
// is not a named capture.
Array.prototype.groups = { a: "b", x: "y", z: "z" };
assert.sameValue("a", "ab".replace(re, "$<a>"));
assert.sameValue("", "ab".replace(re, "$<x>"));
assert.sameValue("", "ab".replace(re, "$<z>"));
Array.prototype.groups = undefined;
