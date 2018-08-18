import { StorageArea, storage as defaultArea } from "std:async-local-storage";
import { assertArrayCustomEquals } from "./equality-asserters.js";

export function testWithArea(testFn, description) {
  promise_test(t => {
    const area = new StorageArea(description);
    t.add_cleanup(t => area.clear());

    return testFn(area, t);
  }, description);
}

export function testWithDefaultArea(testFn, description) {
  promise_test(t => {
    t.add_cleanup(t => defaultArea.clear());

    return testFn(defaultArea, t);
  }, description);
}

// These two functions take a key/value and use them to test
// set()/get()/delete()/has()/keys()/values()/entries(). The keyEqualityAsserter should be a
// function from ./equality-asserters.js.

export function testVariousMethodsWithDefaultArea(label, key, value, keyEqualityAsserter) {
  testWithDefaultArea(testVariousMethodsInner(key, value, keyEqualityAsserter), label);
}

export function testVariousMethods(label, key, value, keyEqualityAsserter) {
  testWithArea(testVariousMethodsInner(key, value, keyEqualityAsserter), label);
}

function testVariousMethodsInner(key, value, keyEqualityAsserter) {
  return async area => {
    await assertPromiseEquals(area.set(key, value), undefined, "set()", "undefined");

    await assertPromiseEquals(area.get(key), value, "get()", "the set value");
    await assertPromiseEquals(area.has(key), true, "has()", "true");

    const keysPromise = area.keys();
    assertIsPromise(keysPromise, "keys()");
    assertArrayCustomEquals(await keysPromise, [key], keyEqualityAsserter, "keys() must have the key");

    const valuesPromise = area.values();
    assertIsPromise(valuesPromise);
    assert_array_equals(await valuesPromise, [value], "values() must have the value");

    const entriesPromise = area.entries();
    assertIsPromise(entriesPromise, "entries()");
    const entries = await entriesPromise;
    assert_true(Array.isArray(entries), "entries() must give an array");
    assert_equals(entries.length, 1, "entries() must have only one value");
    assert_true(Array.isArray(entries[0]), "entries() 0th element must be an array");
    assert_equals(entries[0].length, 2, "entries() 0th element must have 2 elements");
    keyEqualityAsserter(entries[0][0], key, "entries() 0th element's 0th element must be the key");
    assert_equals(entries[0][1], value, "entries() 0th element's 1st element must be the value");

    await assertPromiseEquals(area.delete(key), undefined, "delete()", "undefined");

    await assertPromiseEquals(area.get(key), undefined, "get()", "undefined after deleting");
    await assertPromiseEquals(area.has(key), false, "has()", "false after deleting");
  };
}

async function assertPromiseEquals(promise, expected, label, expectedLabel) {
  assertIsPromise(promise, label);
  assert_equals(await promise, expected, label + " must fulfill with " + expectedLabel);
}

function assertIsPromise(promise, label) {
  assert_equals(promise.constructor, Promise, label + " must return a promise");
}
