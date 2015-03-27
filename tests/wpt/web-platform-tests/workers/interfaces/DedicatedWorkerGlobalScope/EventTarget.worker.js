importScripts("/resources/testharness.js");

test(function() {
    var i = 0;
    addEventListener("message", this.step_func(function listener(evt) {
        ++i;
        removeEventListener("message", listener, true);
    }), true);
    self.dispatchEvent(new Event("message"));
    self.dispatchEvent(new Event("message"));
    assert_equals(i, 1);
}, "removeEventListener");

test(function() {
    addEventListener("message", this.step_func(function(evt) {
        assert_equals(evt.target, self);
    }), true);
    self.dispatchEvent(new Event("message"));
}, "target");

done();
