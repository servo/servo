// Copyright (C) 2016 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Regular expression literals should not compare as equal even if they appear in the same source position.
esid: sec-regular-expression-literals-runtime-semantics-evaluation
---*/

function makeRegExp() {
  return /(?:)/;
}

assert.notSameValue(makeRegExp(), makeRegExp());

var values = [];
for (var i = 0; i < 2; ++i) {
  values[i] = /(?:)/;
}

assert.notSameValue(values[0], values[1]);
