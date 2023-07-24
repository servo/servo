// META: title=StorageManager: result of persist() matches result of persisted()

promise_test(async t => {
    var persistResult = await navigator.storage.persist();
    assert_equals(typeof persistResult, 'boolean', persistResult + ' should be boolean');
    var persistedResult = await navigator.storage.persisted();
    assert_equals(typeof persistedResult, 'boolean', persistedResult + ' should be boolean');
    assert_equals(persistResult, persistedResult);
}, 'navigator.storage.persist() resolves to a value that matches navigator.storage.persisted()');
