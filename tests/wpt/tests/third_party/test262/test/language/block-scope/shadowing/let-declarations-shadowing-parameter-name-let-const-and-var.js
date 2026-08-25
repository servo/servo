// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    let declarations shadowing parameter name, let, const and var
---*/
function fn(a) {
  let b = 1;
  var c = 1;
  const d = 1;
  {
    let a = 2;
    let b = 2;
    let c = 2;
    let d = 2;
    assert.sameValue(a, 2);
    assert.sameValue(b, 2);
    assert.sameValue(c, 2);
    assert.sameValue(d, 2);
  }
}
fn(1);

