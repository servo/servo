// Step 1.
test(function() {
    assert_throws_dom("TypeMismatchError", function() {
        self.crypto.getRandomValues(new Float32Array(6))
    }, "Float32Array")
    assert_throws_dom("TypeMismatchError", function() {
        self.crypto.getRandomValues(new Float64Array(6))
    }, "Float64Array")

    assert_throws_dom("TypeMismatchError", function() {
        self.crypto.getRandomValues(new Float32Array(65537))
    }, "Float32Array (too long)")
    assert_throws_dom("TypeMismatchError", function() {
        self.crypto.getRandomValues(new Float64Array(65537))
    }, "Float64Array (too long)")
}, "Float arrays")

var arrays = {
    'Int8Array': Int8Array,
    'Int16Array': Int16Array,
    'Int32Array': Int32Array,
    'Uint8Array': Uint8Array,
    'Uint8ClampedArray': Uint8ClampedArray,
    'Uint16Array': Uint16Array,
    'Uint32Array': Uint32Array,
};

test(function() {
    for (var array in arrays) {
        assert_equals(self.crypto.getRandomValues(new arrays[array](8)).constructor,
                      arrays[array], "crypto.getRandomValues(new " + array + "(8))")
    }
}, "Integer array")

test(function() {
    for (var array in arrays) {
        var maxlength = 65536 / (arrays[array].BYTES_PER_ELEMENT);
        assert_throws_dom("QuotaExceededError", function() {
            self.crypto.getRandomValues(new arrays[array](maxlength + 1))
        }, "crypto.getRandomValues length over 65536")
    }
}, "Large length")

test(function() {
    for (var array in arrays) {
        assert_true(self.crypto.getRandomValues(new arrays[array](0)).length == 0)
    }
}, "Null arrays")
