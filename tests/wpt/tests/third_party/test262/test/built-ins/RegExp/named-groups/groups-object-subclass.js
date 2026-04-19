// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Test the groups object on RegExp subclass results that have their own.
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

class FakeRegExp extends RegExp {
  exec(subject) {
    const fakeResult = ["ab", "a"];
    fakeResult.index = 0;
    fakeResult.groups = { a: "b" };
    Object.getPrototypeOf(fakeResult.groups).b = "c";
    return fakeResult;
  }
};

const re = new FakeRegExp();
const result = re.exec("ab");
assert.sameValue(Object.getPrototypeOf(result), Array.prototype);
assert(result.hasOwnProperty("groups"));
assert.sameValue("b", result.groups.a);
assert.sameValue("b", "ab".replace(re, "$<a>"));
assert.sameValue("c", "ab".replace(re, "$<b>"));
