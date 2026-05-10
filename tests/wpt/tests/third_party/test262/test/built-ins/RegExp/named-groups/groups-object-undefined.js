// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The groups object is created unconditionally.
includes: [propertyHelper.js]
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

const re = /./;
const result = re.exec("a");
assert.sameValue(Object.getPrototypeOf(result), Array.prototype);
assert(result.hasOwnProperty("groups"));
assert.sameValue("a", result[0]);
assert.sameValue(0, result.index);
assert.sameValue(undefined, result.groups);
verifyProperty(result, "groups", {
  writable: true,
  enumerable: true,
  configurable: true,
});

Array.prototype.groups = { a: "b" };
assert.sameValue("$<a>", "a".replace(re, "$<a>"));
Array.prototype.groups = undefined;
