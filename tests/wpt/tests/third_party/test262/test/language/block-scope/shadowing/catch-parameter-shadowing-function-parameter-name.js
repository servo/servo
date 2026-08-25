// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    catch parameter shadowing function parameter name
---*/
function fn(a) {
  try {
    throw 'stuff1';
  } catch (a) {
    assert.sameValue(a, 'stuff1');
    // catch parameter shadowing function parameter name
    a = 2;
    assert.sameValue(a, 2);
  }
}
fn(1);

