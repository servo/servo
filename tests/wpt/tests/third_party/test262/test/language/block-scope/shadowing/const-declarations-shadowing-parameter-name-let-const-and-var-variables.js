// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    const declarations shadowing parameter name, let, const and var variables
---*/
function fn(a) {
  let b = 1;
  var c = 1;
  const d = 1;
  {
    const a = 2;
    const b = 2;
    const c = 2;
    const d = 2;
    assert.sameValue(a, 2);
    assert.sameValue(b, 2);
    assert.sameValue(c, 2);
    assert.sameValue(d, 2);
  }

  assert.sameValue(a, 1);
  assert.sameValue(b, 1);
  assert.sameValue(c, 1);
  assert.sameValue(d, 1);
}
fn(1);

