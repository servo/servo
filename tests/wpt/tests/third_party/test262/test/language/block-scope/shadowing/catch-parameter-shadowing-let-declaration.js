// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    catch parameter shadowing let declaration
---*/
{
  let a = 3;
  try {
    throw 'stuff2';
  } catch (a) {
    assert.sameValue(a, 'stuff2');
    // catch parameter shadowing let declaration
    a = 4;
    assert.sameValue(a, 4);
  }
  assert.sameValue(a, 3);
}

