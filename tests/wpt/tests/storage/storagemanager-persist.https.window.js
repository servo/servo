// META: title=StorageManager: persist()

promise_test(function() {
    var promise = navigator.storage.persist();
    assert_true(promise instanceof Promise,
        'navigator.storage.persist() returned a Promise.');
    return promise.then(function(result) {
        assert_equals(typeof result, 'boolean', result + ' should be boolean');
    });
}, 'navigator.storage.persist() returns a promise that resolves.');
