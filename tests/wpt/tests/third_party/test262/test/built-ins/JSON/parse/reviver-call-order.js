// Copyright 2019 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-internalizejsonproperty
description: JSON.parse reviver call order
features: [for-in-order]
includes: [compareArray.js]
---*/

var calls = [];
function reviver(name, val) {
  calls.push(name);
  return val;
}

JSON.parse('{"p1":0,"p2":0,"p1":0,"2":0,"1":0}', reviver);

// The empty string is the _rootName_ in JSON.parse
assert.compareArray(calls, ['1', '2', 'p1', 'p2', '']);
