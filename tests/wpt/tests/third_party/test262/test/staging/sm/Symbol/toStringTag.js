/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
// ES6 19.1.3.6 Object.prototype.toString ( )
function testToString() {
    var tests = [
        [undefined, "[object Undefined]"],
        [null, "[object Null]"],
        [[], "[object Array]"],
        [new String("abc"), "[object String]"],
        [(function () {return arguments;})(), "[object Arguments]"],
        [(function () {"use strict"; return arguments;})(), "[object Arguments]"],
        [function() {}, "[object Function]"],
        [new Error("abc"), "[object Error]"],
        [true, "[object Boolean]"],
        [5, "[object Number]"],
        [new Date(), "[object Date]"],
        [/regexp/, "[object RegExp]"],
        [{[Symbol.toStringTag]: "abc"}, "[object abc]"],
        [Object.create(JSON), "[object JSON]"],
        [Object.create(new Number), "[object Object]"],
        [Object.create(new Number, {[Symbol.toStringTag]: {value: "abc"}}), "[object abc]"],
        [(function() { var x = new Number(); x[Symbol.toStringTag] = "abc"; return x; })(), "[object abc]"],
        [[], "[object Array]"]
    ];

    // Testing if the values are obtained the right way.
    for (let [value, expected] of tests) {
        let result = Object.prototype.toString.call(value);
        assert.sameValue(result, expected);
    }
}
testToString();

function testProxy() {
    var count = 0;
    var metaHandler = new Proxy({}, {
        get(target, property, receiver) {
            assert.sameValue(property, "get");

            return function(target, property, receiver) {
                assert.sameValue(property, Symbol.toStringTag);
                count++;
                return undefined;
            }
        }
    });

    assert.sameValue(Object.prototype.toString.call(new Proxy({}, metaHandler)), "[object Object]")
    assert.sameValue(Object.prototype.toString.call(new Proxy(new Date, metaHandler)), "[object Object]")
    assert.sameValue(Object.prototype.toString.call(new Proxy([], metaHandler)), "[object Array]")
    assert.sameValue(Object.prototype.toString.call(new Proxy(function() {}, metaHandler)), "[object Function]")
    var {proxy, revoke} = Proxy.revocable({}, metaHandler);
    revoke();
    assert.throws(TypeError, () => Object.prototype.toString.call(proxy));

    assert.sameValue(count, 4);
}
testProxy();

// Tests the passed objects toStringTag values and ensures it's
// desc is writable: false, enumerable: false, configurable: true
function testDefault(object, expected) {
    let desc = Object.getOwnPropertyDescriptor(object, Symbol.toStringTag);
    assert.sameValue(desc.value, expected);
    assert.sameValue(desc.writable, false);
    assert.sameValue(desc.enumerable, false);
    assert.sameValue(desc.configurable, true);
}

// ES6 19.4.3.5 Symbol.prototype [ @@toStringTag ]
testDefault(Symbol.prototype, "Symbol");

// ES6 20.2.1.9 Math [ @@toStringTag ]
testDefault(Math, "Math");

// ES6 21.1.5.2.2 %StringIteratorPrototype% [ @@toStringTag ]
testDefault(""[Symbol.iterator]().__proto__, "String Iterator")

// ES6 22.1.5.2.2 %ArrayIteratorPrototype% [ @@toStringTag ]
testDefault([][Symbol.iterator]().__proto__, "Array Iterator")

// ES6 22.2.3.31 get %TypedArray%.prototype [ @@toStringTag ]
function testTypedArray() {
    let ta = (new Uint8Array(0)).__proto__.__proto__;
    let desc = Object.getOwnPropertyDescriptor(ta, Symbol.toStringTag);
    assert.sameValue(desc.enumerable, false);
    assert.sameValue(desc.configurable, true);
    assert.sameValue(desc.set, undefined);

    let get = desc.get;
    assert.sameValue(get.name, "get [Symbol.toStringTag]");
    assert.sameValue(get.call(3.14), undefined);
    assert.sameValue(get.call({}), undefined);
    assert.sameValue(get.call(ta), undefined);

    let types = [
        Int8Array,
        Uint8Array,
        Int16Array,
        Uint16Array,
        Int32Array,
        Uint32Array,
        Float32Array,
        Float64Array
    ];

    for (let type of types) {
        let array = new type(0);
        assert.sameValue(get.call(array), type.name);
        assert.sameValue(Object.prototype.toString.call(array), `[object ${type.name}]`);
    }
}
testTypedArray();

// ES6 23.1.3.13 Map.prototype [ @@toStringTag ]
testDefault(Map.prototype, "Map");

// ES6 23.1.5.2.2 %MapIteratorPrototype% [ @@toStringTag ]
testDefault(new Map()[Symbol.iterator]().__proto__, "Map Iterator")

// ES6 23.2.3.12 Set.prototype [ @@toStringTag ]
testDefault(Set.prototype, "Set");

// ES6 23.2.5.2.2 %SetIteratorPrototype% [ @@toStringTag ]
testDefault(new Set()[Symbol.iterator]().__proto__, "Set Iterator")

// ES6 23.3.3.6 WeakMap.prototype [ @@toStringTag ]
testDefault(WeakMap.prototype, "WeakMap");

// ES6 23.4.3.5 WeakSet.prototype [ @@toStringTag ]
testDefault(WeakSet.prototype, "WeakSet");

// ES6 24.1.4.4 ArrayBuffer.prototype [ @@toStringTag ]
testDefault(ArrayBuffer.prototype, "ArrayBuffer");

// ES6 24.2.4.21 DataView.prototype[ @@toStringTag ]
testDefault(DataView.prototype, "DataView");

// ES6 24.3.3 JSON [ @@toStringTag ]
testDefault(JSON, "JSON");

// ES6 25.2.3.3 GeneratorFunction.prototype [ @@toStringTag ]
testDefault(function* () {}.constructor.prototype, "GeneratorFunction");

// ES6 25.3.1.5 Generator.prototype [ @@toStringTag ]
testDefault(function* () {}().__proto__.__proto__, "Generator");

// ES6 25.4.5.4 Promise.prototype [ @@toStringTag ]
testDefault(Promise.prototype, "Promise");

// AsyncFunction.prototype [ @@toStringTag ]
testDefault(async function() {}.constructor.prototype, "AsyncFunction");

