/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/assertThrowsValue.js, deepEqual.js]
description: |
  pending
esid: pending
---*/

// Array.from throws if the argument is undefined or null.
assert.throws(TypeError, () => Array.from());
assert.throws(TypeError, () => Array.from(undefined));
assert.throws(TypeError, () => Array.from(null));

// Array.from throws if an element can't be defined on the new object.
function ObjectWithReadOnlyElement() {
    Object.defineProperty(this, "0", {value: null});
    this.length = 0;
}
ObjectWithReadOnlyElement.from = Array.from;
assert.deepEqual(ObjectWithReadOnlyElement.from([]), new ObjectWithReadOnlyElement);
assert.throws(TypeError, () => ObjectWithReadOnlyElement.from([1]));

// The same, but via preventExtensions.
function InextensibleObject() {
    Object.preventExtensions(this);
}
InextensibleObject.from = Array.from;
assert.throws(TypeError, () => InextensibleObject.from([1]));

// We will now test this property, that Array.from throws if the .length can't
// be assigned, using several different kinds of object.
var obj;
function init(self) {
    obj = self;
    self[0] = self[1] = self[2] = self[3] = 0;
}

function testUnsettableLength(C, Exc) {
    if (Exc === undefined)
        Exc = TypeError;  // the usual expected exception type
    C.from = Array.from;

    obj = null;
    assert.throws(Exc, () => C.from([]));
    assert.sameValue(obj instanceof C, true);
    for (var i = 0; i < 4; i++)
        assert.sameValue(obj[0], 0);

    obj = null;
    assert.throws(Exc, () => C.from([0, 10, 20, 30]));
    assert.sameValue(obj instanceof C, true);
    for (var i = 0; i < 4; i++)
        assert.sameValue(obj[i], i * 10);
}

// Array.from throws if the new object's .length can't be assigned because
// there is no .length and the object is inextensible.
function InextensibleObject4() {
    init(this);
    Object.preventExtensions(this);
}
testUnsettableLength(InextensibleObject4);

// Array.from throws if the new object's .length can't be assigned because it's
// read-only.
function ObjectWithReadOnlyLength() {
    init(this);
    Object.defineProperty(this, "length", {configurable: true, writable: false, value: 4});
}
testUnsettableLength(ObjectWithReadOnlyLength);

// The same, but using a builtin type.
Uint8Array.from = Array.from;
assert.throws(TypeError, () => Uint8Array.from([]));

// Array.from throws if the new object's .length can't be assigned because it
// inherits a readonly .length along the prototype chain.
function ObjectWithInheritedReadOnlyLength() {
    init(this);
}
Object.defineProperty(ObjectWithInheritedReadOnlyLength.prototype,
                      "length",
                      {configurable: true, writable: false, value: 4});
testUnsettableLength(ObjectWithInheritedReadOnlyLength);

// The same, but using an object with a .length getter but no setter.
function ObjectWithGetterOnlyLength() {
    init(this);
    Object.defineProperty(this, "length", {configurable: true, get: () => 4});
}
testUnsettableLength(ObjectWithGetterOnlyLength);

// The same, but with a setter that throws.
function ObjectWithThrowingLengthSetter() {
    init(this);
    Object.defineProperty(this, "length", {
        configurable: true,
        get: () => 4,
        set: () => { throw new RangeError("surprise!"); }
    });
}
testUnsettableLength(ObjectWithThrowingLengthSetter, RangeError);

// Array.from throws if mapfn is neither callable nor undefined.
assert.throws(TypeError, () => Array.from([3, 4, 5], {}));
assert.throws(TypeError, () => Array.from([3, 4, 5], "also not a function"));
assert.throws(TypeError, () => Array.from([3, 4, 5], null));

// Even if the function would not have been called.
assert.throws(TypeError, () => Array.from([], JSON));

// If mapfn is not undefined and not callable, the error happens before anything else.
// Before calling the constructor, before touching the arrayLike.
var log = "";
function C() {
    log += "C";
    obj = this;
}
var p = new Proxy({}, {
    has: function () { log += "1"; },
    get: function () { log += "2"; },
    getOwnPropertyDescriptor: function () { log += "3"; }
});
assert.throws(TypeError, () => Array.from.call(C, p, {}));
assert.sameValue(log, "");

// If mapfn throws, the new object has already been created.
var arrayish = {
    get length() { log += "l"; return 1; },
    get 0() { log += "0"; return "q"; }
};
log = "";
var exc = {surprise: "ponies"};
assertThrowsValue(() => Array.from.call(C, arrayish, () => { throw exc; }), exc);
assert.sameValue(log, "lC0");
assert.sameValue(obj instanceof C, true);

// It's a TypeError if the @@iterator property is a primitive (except null and undefined).
for (var primitive of ["foo", 17, Symbol(), true]) {
    assert.throws(TypeError, () => Array.from({[Symbol.iterator] : primitive}));
}
assert.deepEqual(Array.from({[Symbol.iterator]: null}), []);
assert.deepEqual(Array.from({[Symbol.iterator]: undefined}), []);

// It's a TypeError if the iterator's .next() method returns a primitive.
for (var primitive of [undefined, null, 17]) {
    assert.throws(
        TypeError,
        () => Array.from({
            [Symbol.iterator]() {
                return {next() { return primitive; }};
            }
        })
    );
}

