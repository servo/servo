// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    const declaration shadowing catch parameter
---*/
function fn() {
  var a = 1;
  try {
    throw 'stuff3';
  } catch (a) {
    {
      // const declaration shadowing catch parameter
      const a = 3;
      assert.sameValue(a, 3);
    }
    assert.sameValue(a, 'stuff3');
  }
  assert.sameValue(a, 1);
}
fn();

