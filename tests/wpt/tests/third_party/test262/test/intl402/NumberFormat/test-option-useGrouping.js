// Copyright 2012 Mozilla Corporation. All rights reserved.
// Copyright 2022 Apple Inc. All rights reserved.
// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.1.1_34
description: Tests that the option useGrouping is processed correctly.
info: |
  The "Intl.NumberFormat v3" proposal contradicts the behavior required by the
  latest revision of ECMA402.
author: Norbert Lindenberg
features: [Intl.NumberFormat-v3]
---*/

function resolveUseGrouping(option) {
  return new Intl.NumberFormat(undefined, { useGrouping: option }).resolvedOptions().useGrouping;
}

for (let string of ["min2", "auto", "always"]) {
  assert.sameValue(resolveUseGrouping(string), string);
}

assert.sameValue(resolveUseGrouping(true), "always");
assert.sameValue(resolveUseGrouping(false), false);
assert.sameValue(resolveUseGrouping(undefined), "auto");
assert.sameValue(resolveUseGrouping("true"), "auto");
assert.sameValue(resolveUseGrouping("false"), "auto");

for (let falsy of [0, null, ""]) {
  assert.sameValue(resolveUseGrouping(falsy), false);
}

for (let invalidOptions of [42, "MIN2", {} , "True",  "TRUE" , "FALSE" , "False" , "Undefined" , "undefined"]) {
  assert.throws(RangeError, function () {
    return new Intl.NumberFormat(undefined, { useGrouping: invalidOptions });
  }, "Throws RangeError when useGrouping value is not supported");
}


