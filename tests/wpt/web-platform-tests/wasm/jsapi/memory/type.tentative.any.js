// META: global=window,dedicatedworker,jsshell
// META: script=/wasm/jsapi/assertions.js

function assert_type(argument) {
    const memory = new WebAssembly.Memory(argument);

    assert_equals(memory.type.minimum, argument.minimum);
    assert_equals(memory.type.maximum, argument.maximum);
    if (argument.shared !== undefined) {
        assert_equals(memory.type.shared, argument.shared);
    }
}

test(() => {
    assert_type({ "minimum": 0 });
}, "Zero initial, no maximum");

test(() => {
    assert_type({ "minimum": 5 });
}, "Non-zero initial, no maximum");

test(() => {
    assert_type({ "minimum": 0, "maximum": 0 });
}, "Zero maximum");

test(() => {
    assert_type({ "minimum": 0, "maximum": 5 });
}, "None-zero maximum");

test(() => {
    assert_type({ "minimum": 0, "maximum": 10,  "shared": false});
}, "non-shared memory");

test(() => {
    assert_type({ "minimum": 0, "maximum": 10, "shared": true});
}, "shared memory");