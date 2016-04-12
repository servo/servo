importScripts('worker-testharness.js');

self.oninstall = function(event) {
    assert_true(event instanceof ExtendableEvent);
    assert_equals(event.type, 'install');
    assert_false(event.cancelable);
    assert_false(event.bubbles);
};
