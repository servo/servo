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

let setLikeTests = [
    {
        setConstructor: TestBindingSetlikeWithPrimitive,
        testValues: ["first", "second", "third", "fourth"],
        memberType: "string"
    },
    {
        setConstructor: TestBindingSetlikeWithInterface,
        testValues: [new TestBinding(), new TestBinding(), new TestBinding(), new TestBinding()],
        memberType: "TestBinding"
    },
];

for (const { setConstructor, testValues, memberType } of setLikeTests) {
    test(function () {
        var s = new setConstructor();
        assert_true(ok(s), `got a ${setConstructor.name} object`);
        testExistence(setConstructor.name, s, setlike_rw_properties);
        assert_equals(s.size, 0, "size of new set should be zero");
        const testValue1 = testValues[0];
        assert_true(!s.has(testValue1), "has() should return false for bogus value");
        s1 = s.add(testValue1);
        assert_equals(s, s1, "set.add() should be a chainable method");
        assert_equals(s.size, 1, "size should be 1");
        assert_true(s.has(testValue1), "has() should return true for the first test value");
        const testValue2 = testValues[1];
        s.add(testValue2);
        assert_equals(s.size, 2, "size should be 2");
        testIndex = 0;
        s.forEach(function (v, k, o) {
            "use strict";
            assert_equals(o, s, "forEach obj is correct");
            assert_equals(k, testValues[testIndex], "forEach set key: " + k + " = " + testValues[testIndex]);
            testIndex += 1;
        });
        assert_equals(testIndex, 2, "forEach ran correct number of times");
        assert_true(s.has(testValue2), "maplike has should return true for second key");
        assert_equals(s.delete(testValue2), true, "maplike deletion should return true");
        assert_equals(s.size, 1, "size should be 1");
        iterable = false;
        for (let e of s) {
            iterable = true;
            assert_equals(e, testValue1, "iterable first array element should be first test key");
        }
        assert_equals(s[Symbol.iterator].length, 0, "@@iterator symbol is correct length");
        assert_equals(s[Symbol.iterator].name, "values", "@@iterator symbol has correct name");
        assert_equals(s[Symbol.iterator], s.values, '@@iterator is an alias for "values"');
        assert_true(ok(iterable), " @@iterator symbol resolved correctly");
        for (let k of s.keys()) {
            assert_equals(k, testValue1, "first element of keys() should be the first test key");
        }
        for (let v of s.values()) {
            assert_equals(v, testValue1, "first element of values() should be the first test value");
        }
        for (let e of s.entries()) {
            assert_equals(e[0], testValue1, "first element of entries() should be the first test value");
            assert_equals(e[1], testValue1, "second element of entries() should be the second test value");
        }
        s.clear();
        assert_equals(s.size, 0, "size should be 0 after clear");
    }, `setlike<${memberType}> - Basic set operations`);

    test(function () {
        // Test this override for forEach
        s = new setConstructor();
        s.add(testValues[0]);
        s.forEach(function (v, k, o) {
            "use strict";
            assert_equals(o, s, "forEach obj is correct");
            assert_equals(this, 5, "'this' value should be correct");
        }, 5);
    }, `setke<${memberType}> - Test 'this' override for 'forEach'`);

    // some iterable test ported to *like interfaces
    test(function () {
        var s = new setConstructor();
        var empty = true;
        s.forEach(function () { empty = false; });
        assert_true(empty);
    }, `Empty setlike<${memberType}>`);

    test(function () {
        var s = new setConstructor();
        function is_iterator(o) {
            return o[Symbol.iterator]() === o;
        }
        assert_true(is_iterator(s.keys()));
        assert_true(is_iterator(s.values()));
        assert_true(is_iterator(s.entries()));
    }, `setlike<${memberType}> are iterators`);

    test(function () {
        var s = new setConstructor();
        s.add(testValues[0]);
        s.add(testValues[1]);
        s.add(testValues[2]);
        assert_array_equals(collect(s.keys()), collect(s.values()));
        assert_array_equals(collect(s.values()), testValues.slice(0, 3));
        var i = 0;
        for (entry of s.entries()) {
            assert_array_equals(entry, [testValues[i], testValues[i]]);
            i += 1;
        }

        s.add(testValues[3]);
        assert_array_equals(collect(s.keys()), collect(s.values()));
        assert_array_equals(collect(s.values()), testValues);
        var i = 0;
        for (entry of s.entries()) {
            assert_array_equals(entry, [testValues[i], testValues[i]]);
            i += 1;
        }
    }, `setlike<${memberType}> - Iterators iterate over values`);
}

test(function () {
    var m = new TestBindingSetlikeWithPrimitive();
    m.add();
    assert_equals(m.size, 1, "set should have 1 entry");
    m.forEach(function (v, k) {
        "use strict";
        assert_equals(typeof k, "string", "key must be a string");
        assert_equals(k, "undefined", "key is the string undefined");
    });
    m.delete();
    assert_equals(m.size, 0, "after deleting key, set should have 0 entries");
}, "setlike<DOMString> - Default arguments for r/w methods is undefined");

let mapLikeTests = [
    {
        mapConstructor: TestBindingMaplikeWithPrimitive,
        testEntries: [["first", 1], ["second", 2], ["third", 3], ["fourth", 4]],
        valueType: "number"
    },
    {
        mapConstructor: TestBindingMaplikeWithInterface,
        testEntries: [
            ["first", new TestBinding()],
            ["second", new TestBinding()],
            ["third", new TestBinding()],
            ["fourth", new TestBinding()],
        ],
        valueType: "TestBinding"
    },
];

for (const { mapConstructor, testEntries, valueType } of mapLikeTests) {
    test(function () {
        m = new mapConstructor();
        assert_true(ok(m), `got a ${mapConstructor.name} object`);
        assert_equals(m.get("test"), undefined, "get(bogusKey) is undefined");
    }, `maplike<string, ${valueType}> - 'get' with a bogus key should return undefined`);

    test(function () {
        // Simple map creation and functionality test
        m = new mapConstructor();
        assert_true(ok(m), `got a ${mapConstructor.name} object`);
        testExistence(mapConstructor.name, m, maplike_rw_properties);
        assert_equals(m.size, 0, "size of new map should be zero");
        let [testKey1, testValue1] = testEntries[0];
        assert_true(!m.has(testKey1), "maplike has should return false for bogus key");
        var m1 = m.set(testKey1, testValue1);
        assert_equals(m, m1, "map.set should be a chainable method");
        assert_equals(m.size, 1, "size should be 1");
        assert_true(m.has(testKey1), "has() should return true for key already added");
        assert_equals(m.get(testKey1), testValue1, "get(testKey1) should return the same value provided to set()");
        let [testKey2, testValue2] = testEntries[1];
        m.set(testKey2, testValue2);
        assert_equals(m.size, 2, "size should be 2");
        testSet = testEntries.slice(0, 2);
        testIndex = 0;
        m.forEach(function (v, k, o) {
            "use strict";
            assert_equals(o, m, "forEach obj is correct");
            assert_equals(k, testSet[testIndex][0], "forEach map key: " + k + " = " + testSet[testIndex][0]);
            assert_equals(v, testSet[testIndex][1], "forEach map value: " + v + " = " + testSet[testIndex][1]);
            testIndex += 1;
        });
        assert_equals(testIndex, 2, "forEach ran correct number of times");
        assert_true(m.has(testKey2), "has() should return true for second test key");
        assert_equals(m.get(testKey2), testValue2, "get(testKey2) should return the same value provided to set()");
        assert_equals(m.delete(testKey2), true, "maplike deletion should return boolean");
        assert_equals(m.size, 1, "size should be 1");
        var iterable = false;
        for (let e of m) {
            iterable = true;
            assert_equals(e[0], testKey1, "iterable's first array element should be the first test key");
            assert_equals(e[1], testValue1, "iterable' second array element should be the first test value");
        }
        assert_equals(m[Symbol.iterator].length, 0, "@@iterator symbol is correct length");
        assert_equals(m[Symbol.iterator].name, "entries", "@@iterator symbol has correct name");
        assert_equals(m[Symbol.iterator], m.entries, '@@iterator is an alias for "entries"');
        assert_true(ok(iterable), "@@iterator symbol resolved correctly");
        for (let k of m.keys()) {
            assert_equals(k, testKey1, "first element of keys() should be the first test key");
        }
        for (let v of m.values()) {
            assert_equals(v, testValue1, "first element of values() should be 1");
        }
        for (let e of m.entries()) {
            assert_equals(e[0], testKey1, "first element of entries() should have the first test key");
            assert_equals(e[1], testValue1, "first element of entries() should have the second test value");
        }
        m.clear();
        assert_equals(m.size, 0, "size should be 0 after clear");
    }, `maplike<string, ${valueType}> - Simple map creation and functionality test`);

    test(function () {
        // Map convenience function test
        m = new mapConstructor();
        assert_true(ok(m), `got a ${mapConstructor.name} object`);
        assert_equals(m.size, 0, "size should be zero");
        assert_true(!m.hasInternal("test"), "hasInternal() should return false for bogus key");
        // It's fine to let getInternal to return 0 if the key doesn't exist
        // because this API can only be used internally in C++ and we'd throw
        // an error if the key doesn't exist.
        //SimpleTest.doesThrow(() => m.getInternal("test"), 0, "MapConvenience: maplike getInternal should throw if the key doesn't exist");
        let [testKey1, testValue1] = testEntries[0];
        m.setInternal(testKey1, testValue1);
        assert_equals(m.size, 1, "size should be 1 after adding first test key/value");
        assert_true(m.hasInternal(testKey1), "hasInternal() should return true");
        assert_equals(m.get(testKey1), testValue1, "get() should return the value set using setInternal()");
        assert_equals(m.getInternal(testKey1), testValue1, "getInternal() should return the value set using setInternal()");
        let [testKey2, testValue2] = testEntries[1];
        m.setInternal(testKey2, testValue2);
        assert_equals(m.size, 2, "size should be 2");
        assert_true(m.hasInternal(testKey2), "hasInternal() should return true for newly added second test key");
        assert_equals(m.get(testKey2), testValue2, "get(testKey2) should return the value set using setInternal");
        assert_equals(m.getInternal(testKey2), testValue2, "getInternal(testKey2) should return the value set using setInternal()");
        assert_equals(m.deleteInternal(testKey2), true, "deleteInternal should return true when deleting existing key");
        assert_equals(m.size, 1, "size should be 1");
        m.clearInternal();
        assert_equals(m.size, 0, "size should be 0 after clearInternal");
    }, `Convenience methods for maplike<string, ${valueType}>`);

    // JS implemented map creation convenience function test
    test(function () {
        // Test this override for forEach
        m = new mapConstructor();
        m.set(testEntries[0][0], testEntries[1][1]);
        m.forEach(function (v, k, o) {
            "use strict";
            assert_equals(o, m, "forEach obj is correct");
            assert_equals(this, 5, "'this' value should be correct");
        }, 5);
    }, `maplike<string, ${valueType}> - Test 'this' override for 'forEach'`);

    // some iterable test ported to *like interfaces
    test(function () {
        var t = new mapConstructor();
        var empty = true;
        t.forEach(function () { empty = false; });
        assert_true(empty);
    }, `maplike<string, ${valueType}> - Empty maplike`);

    test(function () {
        var t = new mapConstructor();
        function is_iterator(o) {
            return o[Symbol.iterator]() === o;
        }
        assert_true(is_iterator(t.keys()));
        assert_true(is_iterator(t.values()));
        assert_true(is_iterator(t.entries()));
    }, `maplike<string, ${valueType}> - Maplike are iterators`);

    test(function () {
        var t = new mapConstructor();
        t.set(testEntries[0][0], testEntries[0][1]);
        t.set(testEntries[1][0], testEntries[1][1]);
        t.set(testEntries[2][0], testEntries[2][1]);
        assert_array_equals(collect(t.keys()), [testEntries[0][0], testEntries[1][0], testEntries[2][0]]);
        assert_array_equals(collect(t.values()), [testEntries[0][1], testEntries[1][1], testEntries[2][1]]);
        var expected = testEntries.slice(0, 3);
        var i = 0;
        for (entry of t.entries()) {
            assert_array_equals(entry, expected[i++]);
        }

        t.set(testEntries[3][0], testEntries[3][1]);
        assert_array_equals(collect(t.keys()),  testEntries.map(entry => entry[0]));
        assert_array_equals(collect(t.values()), testEntries.map(entry => entry[1]));
        var expected = testEntries.slice();
        var i = 0;
        for (entry of t.entries()) {
            assert_array_equals(entry, expected[i++]);
        }
    }, `maplike<string, ${valueType}> - Maplike iteratable over key/value pairs`);
}

test(function () {
    // Test defaulting arguments on maplike to undefined
    m = new TestBindingMaplikeWithPrimitive();
    m.set();
    assert_equals(m.size, 1, "should have 1 entry");
    m.forEach(function (v, k) {
        "use strict";
        assert_equals(typeof k, "string", "defaulted key must be a string");
        assert_equals(k, "undefined", "defaulted key must be the string undefined");
        assert_equals(v, 0, "defaulted value must be 0");
    });
    assert_equals(m.get(), 0, "no argument to get() returns correct value");
    m.delete();
    assert_equals(m.size, 0, "MapArgsDefault: should have 0 entries");
}, "maplike<DOMString, number> - Default arguments for r/w methods is undefined");

