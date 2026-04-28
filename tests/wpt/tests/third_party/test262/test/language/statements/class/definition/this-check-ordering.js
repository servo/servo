// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class this check ordering
---*/
var baseCalled = 0;
class Base {
  constructor() { baseCalled++ }
}

var fCalled = 0;
function f() { fCalled++; return 3; }

class Subclass1 extends Base {
  constructor() {
    baseCalled = 0;
    super();
    assert.sameValue(baseCalled, 1, "The value of `baseCalled` is `1`");
    var obj = this;

    var exn = null;
    baseCalled = 0;
    fCalled = 0;
    try {
      super(f());
    } catch (e) { exn = e; }
    assert.sameValue(
      exn instanceof ReferenceError,
      true,
      "The result of `exn instanceof ReferenceError` is `true`"
    );
    assert.sameValue(fCalled, 1, "The value of `fCalled` is `1`");
    assert.sameValue(baseCalled, 1, "The value of `baseCalled` is `1`");
    assert.sameValue(this, obj, "`this` is `obj`");

    exn = null;
    baseCalled = 0;
    fCalled = 0;
    try {
      super(super(), f());
    } catch (e) { exn = e; }
    assert.sameValue(
      exn instanceof ReferenceError,
      true,
      "The result of `exn instanceof ReferenceError` is `true`"
    );
    assert.sameValue(fCalled, 0, "The value of `fCalled` is `0`");
    assert.sameValue(baseCalled, 1, "The value of `baseCalled` is `1`");
    assert.sameValue(this, obj, "`this` is `obj`");

    exn = null;
    baseCalled = 0;
    fCalled = 0;
    try {
      super(f(), super());
    } catch (e) { exn = e; }
    assert.sameValue(
      exn instanceof ReferenceError,
      true,
      "The result of `exn instanceof ReferenceError` is `true`"
    );
    assert.sameValue(fCalled, 1, "The value of `fCalled` is `1`");
    assert.sameValue(baseCalled, 1, "The value of `baseCalled` is `1`");
    assert.sameValue(this, obj, "`this` is `obj`");
  }
}

new Subclass1();
