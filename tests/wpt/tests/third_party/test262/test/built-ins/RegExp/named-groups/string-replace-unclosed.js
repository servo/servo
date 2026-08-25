// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  A missing > following $< means that the $< is taken literally
  in a replacement string in the context of named capture substitution.
esid: sec-getsubstitution
features: [regexp-named-groups]
---*/

let source = "(?<fst>.)(?<snd>.)|(?<thd>x)";
for (let flags of ["", "u"]) {
  let re = new RegExp(source, flags);
  assert.sameValue("$<sndcd", "abcd".replace(re, "$<snd"));
}
for (let flags of ["g", "gu"]) {
  let re = new RegExp(source, flags);
  assert.sameValue("$<snd$<snd", "abcd".replace(re, "$<snd"));
}
