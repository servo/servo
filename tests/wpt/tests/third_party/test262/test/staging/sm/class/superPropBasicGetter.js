// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class base {
    constructor() {}

    getValue() {
        return this._prop;
    }

    setValue(v) {
        this._prop = v;
    }
}

class derived extends base {
    constructor() { super(); }

    get a() { return super.getValue(); }
    set a(v) { super.setValue(v); }

    get b() { return eval('super.getValue()'); }
    set b(v) { eval('super.setValue(v);'); }

    test() {
        this.a = 15;
        assert.sameValue(this.a, 15);

        assert.sameValue(this.b, 15);
        this.b = 30;
        assert.sameValue(this.b, 30);
    }
}

var derivedInstance = new derived();
derivedInstance.test();

