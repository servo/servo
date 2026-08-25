// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var symbols = [
    Symbol(), Symbol("iterator"), Symbol.for("iterator"), Symbol.iterator
];

for (var sym of symbols) {
    var key = {
        toString() { return sym; }
    };

    // Test that ToPropertyKey can return a symbol in each of the following
    // contexts.

    // Computed property names.
    var obj = {[key]: 13};
    var found = Reflect.ownKeys(obj);
    assert.sameValue(found.length, 1);
    assert.sameValue(found[0], sym);

    // Computed accessor property names.
    var obj2 = {
        get [key]() { return "got"; },
        set [key](v) { this.v = v; }
    };
    assert.sameValue(obj2[sym], "got");
    obj2[sym] = 33;
    assert.sameValue(obj2.v, 33);

    // Getting and setting properties.
    assert.sameValue(obj[key], 13);
    obj[key] = 19;
    assert.sameValue(obj[sym], 19);
    (function () { "use strict"; obj[key] = 20; })();
    assert.sameValue(obj[sym], 20);
    obj[key]++;
    assert.sameValue(obj[sym], 21);

    // Getting properties of primitive values.
    Number.prototype[sym] = "success";
    assert.sameValue(Math.PI[key], "success");
    delete Number.prototype[sym];

    // Getting a super property.
    class X {
        [sym]() { return "X"; }
    }
    class Y extends X {
        [sym]() { return super[key]() + "Y"; }
    }
    var y = new Y();
    assert.sameValue(y[sym](), "XY");

    // Setting a super property.
    class Z {
        set [sym](v) {
            this.self = this;
            this.value = v;
        }
    }
    class W extends Z {
        set [sym](v) {
            this.isW = true;
            super[key] = v;
        }
    }
    var w = new W();
    w[key] = "ok";
    assert.sameValue(w.self, w);
    assert.sameValue(w.value, "ok");
    assert.sameValue(w.isW, true);

    // Deleting properties.
    obj = {[sym]: 1};
    assert.sameValue(delete obj[key], true);
    assert.sameValue(sym in obj, false);

    // LHS of `in` expressions.
    assert.sameValue(key in {iterator: 0}, false);
    assert.sameValue(key in {[sym]: 0}, true);

    // Methods of Object and Object.prototype
    obj = {};
    Object.defineProperty(obj, key, {value: "ok", enumerable: true});
    assert.sameValue(obj[sym], "ok");
    assert.sameValue(obj.hasOwnProperty(key), true);
    assert.sameValue(obj.propertyIsEnumerable(key), true);
    var desc = Object.getOwnPropertyDescriptor(obj, key);
    assert.sameValue(desc.value, "ok");
}

