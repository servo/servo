// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    catch parameter shadowing var variable
---*/
function fn() {
  var a = 1;
  try {
    throw 'stuff3';
  } catch (a) {
    // catch parameter shadowing var variable
    assert.sameValue(a, 'stuff3');
  }
  assert.sameValue(a, 1);
}
fn();

