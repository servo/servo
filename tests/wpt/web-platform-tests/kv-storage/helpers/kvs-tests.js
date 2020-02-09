import defaultArea, { StorageArea } from "std:kv-storage";
import { assertAsyncIteratorEquals, assertAsyncIteratorCustomEquals } from "./equality-asserters.js";

// Used when we're manually creating the database, and so the IDB helpers also want to clean it up.
// If we used testWithArea, then the IDB helpers would time out in their cleanup steps when they
// fail to delete the already-deleted database.
export function testWithAreaNoCleanup(testFn, description) {
  promise_test(t => {
    const area = new StorageArea(description);

    return testFn(area, t);
  }, description);
}

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
// set()/get()/delete()/keys()/values()/entries(). The keyEqualityAsserter should be a
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

    const keysIter = area.keys();
    await assertAsyncIteratorCustomEquals(keysIter, [key], keyEqualityAsserter, "keys() must have the key");

    const valuesIter = area.values();
    await assertAsyncIteratorEquals(valuesIter, [value], "values() must have the value");

    const entriesIter = area.entries();

    const entry0 = await entriesIter.next();
    assert_false(entry0.done, "entries() 0th iter-result must not be done");
    assert_true(Array.isArray(entry0.value), "entries() 0th iter-result value must be an array");
    assert_equals(entry0.value.length, 2, "entries() 0th iter-result value must have 2 elements");
    keyEqualityAsserter(entry0.value[0], key, "entries() 0th iter-result value's 0th element must be the key");
    assert_equals(entry0.value[1], value, "entries() 0th iter-result value's 1st element must be the value");

    const entry1 = await entriesIter.next();
    assert_true(entry1.done, "entries() 1st iter-result must be done");
    assert_equals(entry1.value, undefined, "entries() 1st iter-result must have undefined value");

    await assertPromiseEquals(area.delete(key), undefined, "delete()", "undefined");

    await assertPromiseEquals(area.get(key), undefined, "get()", "undefined after deleting");
  };
}

async function assertPromiseEquals(promise, expected, label, expectedLabel) {
  assertIsPromise(promise, label);
  assert_equals(await promise, expected, label + " must fulfill with " + expectedLabel);
}

function assertIsPromise(promise, label) {
  assert_equals(promise.constructor, Promise, label + " must return a promise");
}
