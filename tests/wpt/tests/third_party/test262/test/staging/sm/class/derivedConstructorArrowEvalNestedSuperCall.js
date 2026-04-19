// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
new class extends class { } {
    constructor() {
        (()=>eval("super()"))();
        assert.sameValue(this, eval("this"));
        assert.sameValue(this, (()=>this)());
    }
}();

new class extends class { } {
    constructor() {
        (()=>(()=>super())())();
        assert.sameValue(this, eval("this"));
        assert.sameValue(this, (()=>this)());
    }
}();

new class extends class { } {
    constructor() {
        eval("(()=>super())()");
        assert.sameValue(this, eval("this"));
        assert.sameValue(this, (()=>this)());
    }
}();

new class extends class { } {
    constructor() {
        eval("eval('super()')");
        assert.sameValue(this, eval("this"));
        assert.sameValue(this, (()=>this)());
    }
}();

