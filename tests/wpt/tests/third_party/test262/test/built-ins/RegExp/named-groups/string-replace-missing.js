// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: If the group doesn't exist, replace with the empty string
esid: sec-getsubstitution
features: [regexp-named-groups]
---*/

let source = "(?<fst>.)(?<snd>.)|(?<thd>x)";
for (let flags of ["", "u"]) {
  let re = new RegExp(source, flags);
  assert.sameValue("cd", "abcd".replace(re, "$<42$1>"));
  assert.sameValue("cd", "abcd".replace(re, "$<fth>"));
  assert.sameValue("cd", "abcd".replace(re, "$<$1>"));
  assert.sameValue("cd", "abcd".replace(re, "$<>"));
}
for (let flags of ["g", "gu"]) {
  let re = new RegExp(source, flags);
  assert.sameValue("", "abcd".replace(re, "$<42$1>"));
  assert.sameValue("", "abcd".replace(re, "$<fth>"));
  assert.sameValue("", "abcd".replace(re, "$<$1>"));
  assert.sameValue("", "abcd".replace(re, "$<>"));
}
