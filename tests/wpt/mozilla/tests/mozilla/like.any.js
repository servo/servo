// META: title=Setlike and Maplike bindings

function collect(iter) {
    var collection = [];
    for (element of iter) {
        collection.push(element);
    }
    return collection;
}

// mochitest does not do strict true test
function ok(t, desc) {
    if (t) {
        return true
    }
    return false;
}

// tests ported from https://searchfox.org/mozilla-central/source/dom/bindings/test/test_bug1123516_maplikesetlike.html

var base_properties = [["has", "function", 1],
["entries", "function", 0],
["keys", "function", 0],
["values", "function", 0],
["forEach", "function", 1],
["size", "number"]];
var maplike_properties = base_properties.concat([["set", "function", 2]]);
var rw_properties = [["clear", "function", 0],
["delete", "function", 1]];
var setlike_rw_properties = base_properties.concat(rw_properties).concat([["add", "function", 1]]);
var maplike_rw_properties = maplike_properties.concat(rw_properties).concat([["get", "function", 1]]);
var testExistence = function testExistence(prefix, obj, properties) {
    for (var [name, type, args] of properties) {
        // Properties are somewhere up the proto chain, hasOwnProperty won't work
        assert_not_equals(obj[name], undefined,
            `${prefix} object has property ${name}`);

        assert_equals(typeof obj[name], type,
            `${prefix} object property ${name} is a ${type}`);
        // Check function length
        if (type == "function") {
            assert_equals(obj[name].length, args,
                `${prefix} object property ${name} is length ${args}`);
            assert_equals(obj[name].name, name,
                `${prefix} object method name is ${name}`);
        }

        // Find where property is on proto chain, check for enumerablility there.
        var owner = obj;
        while (owner) {
            var propDesc = Object.getOwnPropertyDescriptor(owner, name);
            if (propDesc) {
                assert_true(ok(propDesc.enumerable),
                    `${prefix} object property ${name} should be enumerable`);
                break;
            }
            owner = Object.getPrototypeOf(owner);
        }
    }
};

test(function () {
    var m = new TestBindingSetlike();
    assert_true(ok(m), "SimpleSet: got a TestBindingSetlike object");
    testExistence("SimpleSet: ", m, setlike_rw_properties);
    assert_equals(m.size, 0, "SimpleSet: size should be zero");
    assert_true(!m.has("test"), "SimpleSet: maplike has should return false");
    m1 = m.add("test");
    assert_equals(m, m1, "SimpleSet: return from set should be map object");
    assert_equals(m.size, 1, "SimpleSet: size should be 1");
    assert_true(m.has("test"), "SimpleSet: maplike has should return true");
    m.add("test2");
    assert_equals(m.size, 2, "SimpleSet: size should be 2");
    testSet = ["test", "test2"];
    testIndex = 0;
    m.forEach(function (v, k, o) {
        "use strict";
        assert_equals(o, m, "SimpleSet: foreach obj is correct");
        assert_equals(k, testSet[testIndex], "SimpleSet: foreach set key: " + k + " = " + testSet[testIndex]);
        testIndex += 1;
    });
    assert_equals(testIndex, 2, "SimpleSet: foreach ran correct number of times");
    assert_true(m.has("test2"), "SimpleSet: maplike has should return true");
    assert_equals(m.delete("test2"), true, "SimpleSet: maplike deletion should return true");
    assert_equals(m.size, 1, "SimpleSet: size should be 1");
    iterable = false;
    for (let e of m) {
        iterable = true;
        assert_equals(e, "test", "SimpleSet: iterable first array element should be key");
    }
    assert_equals(m[Symbol.iterator].length, 0, "SimpleSet: @@iterator symbol is correct length");
    assert_equals(m[Symbol.iterator].name, "values", "SimpleSet: @@iterator symbol has correct name");
    assert_equals(m[Symbol.iterator], m.values, 'SimpleSet: @@iterator is an alias for "values"');
    assert_true(ok(iterable), "SimpleSet: @@iterator symbol resolved correctly");
    for (let k of m.keys()) {
        assert_equals(k, "test", "SimpleSet: first keys element should be 'test'");
    }
    for (let v of m.values()) {
        assert_equals(v, "test", "SimpleSet: first values elements should be 'test'");
    }
    for (let e of m.entries()) {
        assert_equals(e[0], "test", "SimpleSet: Entries first array element should be 'test'");
        assert_equals(e[1], "test", "SimpleSet: Entries second array element should be 'test'");
    }
    m.clear();
    assert_equals(m.size, 0, "SimpleSet: size should be 0 after clear");
}, "Simple set creation and functionality");

test(function () {
    var m = new TestBindingSetlike();
    m.add();
    assert_equals(m.size, 1, "SetArgsDefault: should have 1 entry");
    m.forEach(function (v, k) {
        "use strict";
        assert_equals(typeof k, "string", "SetArgsDefault: key is a string");
        assert_equals(k, "undefined", "SetArgsDefault: key is the string undefined");
    });
    m.delete();
    assert_equals(m.size, 0, "SetArgsDefault: should have 0 entries");
}, "Test defaulting arguments on setlike to undefined");

test(function () {
    // Simple map creation and functionality test
    m = new TestBindingMaplike();
    assert_true(ok(m), "SimpleMap: got a TestBindingMaplike object");
    testExistence("SimpleMap: ", m, maplike_rw_properties);
    assert_equals(m.size, 0, "SimpleMap: size should be zero");
    assert_true(!m.has("test"), "SimpleMap: maplike has should return false");
    assert_equals(m.get("test"), undefined, "SimpleMap: maplike get should return undefined on bogus lookup");
    var m1 = m.set("test", 1);
    assert_equals(m, m1, "SimpleMap: return from set should be map object");
    assert_equals(m.size, 1, "SimpleMap: size should be 1");
    assert_true(m.has("test"), "SimpleMap: maplike has should return true");
    assert_equals(m.get("test"), 1, "SimpleMap: maplike get should return value entered");
    m.set("test2", 2);
    assert_equals(m.size, 2, "SimpleMap: size should be 2");
    testSet = [["test", 1], ["test2", 2]];
    testIndex = 0;
    m.forEach(function (v, k, o) {
        "use strict";
        assert_equals(o, m, "SimpleMap: foreach obj is correct");
        assert_equals(k, testSet[testIndex][0], "SimpleMap: foreach map key: " + k + " = " + testSet[testIndex][0]);
        assert_equals(v, testSet[testIndex][1], "SimpleMap: foreach map value: " + v + " = " + testSet[testIndex][1]);
        testIndex += 1;
    });
    assert_equals(testIndex, 2, "SimpleMap: foreach ran correct number of times");
    assert_true(m.has("test2"), "SimpleMap: maplike has should return true");
    assert_equals(m.get("test2"), 2, "SimpleMap: maplike get should return value entered");
    assert_equals(m.delete("test2"), true, "SimpleMap: maplike deletion should return boolean");
    assert_equals(m.size, 1, "SimpleMap: size should be 1");
    var iterable = false;
    for (let e of m) {
        iterable = true;
        assert_equals(e[0], "test", "SimpleMap: iterable first array element should be key");
        assert_equals(e[1], 1, "SimpleMap: iterable second array element should be value");
    }
    assert_equals(m[Symbol.iterator].length, 0, "SimpleMap: @@iterator symbol is correct length");
    assert_equals(m[Symbol.iterator].name, "entries", "SimpleMap: @@iterator symbol has correct name");
    assert_equals(m[Symbol.iterator], m.entries, 'SimpleMap: @@iterator is an alias for "entries"');
    assert_true(ok(iterable), "SimpleMap: @@iterator symbol resolved correctly");
    for (let k of m.keys()) {
        assert_equals(k, "test", "SimpleMap: first keys element should be 'test'");
    }
    for (let v of m.values()) {
        assert_equals(v, 1, "SimpleMap: first values elements should be 1");
    }
    for (let e of m.entries()) {
        assert_equals(e[0], "test", "SimpleMap: entries first array element should be 'test'");
        assert_equals(e[1], 1, "SimpleMap: entries second array element should be 1");
    }
    m.clear();
    assert_equals(m.size, 0, "SimpleMap: size should be 0 after clear");
}, "Simple map creation and functionality test");

test(function () {
    // Map convenience function test
    m = new TestBindingMaplike();
    assert_true(ok(m), "MapConvenience: got a TestBindingMaplike object");
    assert_equals(m.size, 0, "MapConvenience: size should be zero");
    assert_true(!m.hasInternal("test"), "MapConvenience: maplike hasInternal should return false");
    // It's fine to let getInternal to return 0 if the key doesn't exist
    // because this API can only be used internally in C++ and we'd throw
    // an error if the key doesn't exist.
    //SimpleTest.doesThrow(() => m.getInternal("test"), 0, "MapConvenience: maplike getInternal should throw if the key doesn't exist");
    m.setInternal("test", 1);
    assert_equals(m.size, 1, "MapConvenience: size should be 1");
    assert_true(m.hasInternal("test"), "MapConvenience: maplike hasInternal should return true");
    assert_equals(m.get("test"), 1, "MapConvenience: maplike get should return value entered");
    assert_equals(m.getInternal("test"), 1, "MapConvenience: maplike getInternal should return value entered");
    m.setInternal("test2", 2);
    assert_equals(m.size, 2, "size should be 2");
    assert_true(m.hasInternal("test2"), "MapConvenience: maplike hasInternal should return true");
    assert_equals(m.get("test2"), 2, "MapConvenience: maplike get should return value entered");
    assert_equals(m.getInternal("test2"), 2, "MapConvenience: maplike getInternal should return value entered");
    assert_equals(m.deleteInternal("test2"), true, "MapConvenience: maplike deleteInternal should return true");
    assert_equals(m.size, 1, "MapConvenience: size should be 1");
    m.clearInternal();
    assert_equals(m.size, 0, "MapConvenience: size should be 0 after clearInternal");
}, "Map convenience function test");

// JS implemented map creation convenience function test
test(function () {
    // Test this override for forEach
    m = new TestBindingMaplike();
    m.set("test", 1);
    m.forEach(function (v, k, o) {
        "use strict";
        assert_equals(o, m, "ForEachThisOverride: foreach obj is correct");
        assert_equals(this, 5, "ForEachThisOverride: 'this' value should be correct");
    }, 5);
}, "Test this override for forEach");

test(function () {
    // Test defaulting arguments on maplike to undefined
    m = new TestBindingMaplike();
    m.set();
    assert_equals(m.size, 1, "MapArgsDefault: should have 1 entry");
    m.forEach(function (v, k) {
        "use strict";
        assert_equals(typeof k, "string", "MapArgsDefault: key is a string");
        assert_equals(k, "undefined", "MapArgsDefault: key is the string undefined");
        assert_equals(v, 0, "MapArgsDefault: value is 0");
    });
    assert_equals(m.get(), 0, "MapArgsDefault: no argument to get() returns correct value");
    m.delete();
    assert_equals(m.size, 0, "MapArgsDefault: should have 0 entries");
}, "Test defaulting arguments on maplike to undefined");

// some iterable test ported to *like interfaces
test(function () {
    var t = new TestBindingSetlike();
    var empty = true;
    t.forEach(function () { empty = false; });
    assert_true(empty);
}, "Empty setlike");

test(function () {
    var t = new TestBindingSetlike();
    function is_iterator(o) {
        return o[Symbol.iterator]() === o;
    }
    assert_true(is_iterator(t.keys()));
    assert_true(is_iterator(t.values()));
    assert_true(is_iterator(t.entries()));
}, "Setlike are iterators");

test(function () {
    var t = new TestBindingSetlike();
    t.add("first");
    t.add("second");
    t.add("third");
    assert_array_equals(collect(t.keys()), collect(t.values()));
    assert_array_equals(collect(t.values()), ["first", "second", "third"]);
    var expected = [["first", "first"], ["second", "second"], ["third", "third"]];
    var i = 0;
    for (entry of t.entries()) {
        assert_array_equals(entry, expected[i++]);
    }

    t.add("fourth");
    assert_array_equals(collect(t.keys()), collect(t.values()));
    assert_array_equals(collect(t.values()), ["first", "second", "third", "fourth"]);
    var expected = [["first", "first"], ["second", "second"], ["third", "third"], ["fourth", "fourth"]];
    var i = 0;
    for (entry of t.entries()) {
        assert_array_equals(entry, expected[i++]);
    }
}, "Iterators iterate over values");

test(function () {
    var t = new TestBindingMaplike();
    var empty = true;
    t.forEach(function () { empty = false; });
    assert_true(empty);
}, "Empty maplike");

test(function () {
    var t = new TestBindingMaplike();
    function is_iterator(o) {
        return o[Symbol.iterator]() === o;
    }
    assert_true(is_iterator(t.keys()));
    assert_true(is_iterator(t.values()));
    assert_true(is_iterator(t.entries()));
}, "Maplike are iterators");

test(function () {
    var t = new TestBindingMaplike();
    t.set("first", 0);
    t.set("second", 1);
    t.set("third", 2);
    assert_array_equals(collect(t.keys()), ["first", "second", "third"]);
    assert_array_equals(collect(t.values()), [0, 1, 2]);
    var expected = [["first", 0], ["second", 1], ["third", 2]];
    var i = 0;
    for (entry of t.entries()) {
        assert_array_equals(entry, expected[i++]);
    }

    t.set("fourth", 3);
    assert_array_equals(collect(t.keys()), ["first", "second", "third", "fourth"]);
    assert_array_equals(collect(t.values()), [0, 1, 2, 3]);
    var expected = [["first", 0], ["second", 1], ["third", 2], ["fourth", 3]];
    var i = 0;
    for (entry of t.entries()) {
        assert_array_equals(entry, expected[i++]);
    }
}, "Maplike iterate over key/value pairs");
