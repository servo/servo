/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// Like other primitives, symbols can be treated as objects, using object-like
// syntax: `symbol.prop` or `symbol[key]`.
//
// In ECMAScript spec jargon, this creates a Reference whose base value is a
// primitive Symbol value.

var symbols = [
    Symbol(),
    Symbol("ponies"),
    Symbol.for("sym"),
    Symbol.iterator
];

// Test accessor property, used below.
var gets, sets;
Object.defineProperty(Symbol.prototype, "prop", {
    get: function () {
        "use strict";
        gets++;
        assert.sameValue(typeof this, "symbol");
        assert.sameValue(this, sym);
        return "got";
    },
    set: function (v) {
        "use strict";
        sets++;
        assert.sameValue(typeof this, "symbol");
        assert.sameValue(this, sym);
        assert.sameValue(v, "newvalue");
    }
});

for (var sym of symbols) {
    assert.sameValue(sym.constructor, Symbol);

    // method on Object.prototype
    assert.sameValue(sym.hasOwnProperty("constructor"), false);
    assert.sameValue(sym.toLocaleString(), sym.toString()); // once .toString() exists

    // custom method monkeypatched onto Symbol.prototype
    Symbol.prototype.nonStrictMethod = function (arg) {
        assert.sameValue(arg, "ok");
        assert.sameValue(this instanceof Symbol, true);
        assert.sameValue(this.valueOf(), sym);
        return 13;
    };
    assert.sameValue(sym.nonStrictMethod("ok"), 13);

    // the same, but strict mode
    Symbol.prototype.strictMethod = function (arg) {
        "use strict";
        assert.sameValue(arg, "ok2");
        assert.sameValue(this, sym);
        return 14;
    };
    assert.sameValue(sym.strictMethod("ok2"), 14);

    // getter/setter on Symbol.prototype
    gets = 0;
    sets = 0;
    var propname = "prop";

    assert.sameValue(sym.prop, "got");
    assert.sameValue(gets, 1);
    assert.sameValue(sym[propname], "got");
    assert.sameValue(gets, 2);

    assert.sameValue(sym.prop = "newvalue", "newvalue");
    assert.sameValue(sets, 1);
    assert.sameValue(sym[propname] = "newvalue", "newvalue");
    assert.sameValue(sets, 2);

    // non-existent property
    assert.sameValue(sym.noSuchProp, undefined);
    var noSuchPropName = "nonesuch";
    assert.sameValue(sym[noSuchPropName], undefined);

    // non-existent method
    assert.throws(TypeError, () => sym.noSuchProp());
    assert.throws(TypeError, () => sym[noSuchPropName]());
}

