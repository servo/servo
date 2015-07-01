function run_test() {
    // Step 1.
    test(function() {
        assert_throws("TypeMismatchError", function() {
            self.crypto.getRandomValues(new Float32Array(6))
        }, "Float32Array")
        assert_throws("TypeMismatchError", function() {
            self.crypto.getRandomValues(new Float64Array(6))
        }, "Float64Array")

        assert_throws("TypeMismatchError", function() {
            self.crypto.getRandomValues(new Float32Array(65537))
        }, "Float32Array (too long)")
        assert_throws("TypeMismatchError", function() {
            self.crypto.getRandomValues(new Float64Array(65537))
        }, "Float64Array (too long)")
    }, "Float arrays")

    test(function() {
        assert_equals(self.crypto.getRandomValues(new Int8Array(8)).constructor,
                      Int8Array, "crypto.getRandomValues(new Int8Array(8))")
        assert_equals(self.crypto.getRandomValues(new Uint8Array(8)).constructor,
                      Uint8Array, "crypto.getRandomValues(new Uint8Array(8))")

        assert_equals(self.crypto.getRandomValues(new Int16Array(8)).constructor,
                      Int16Array, "crypto.getRandomValues(new Int16Array(8))")
        assert_equals(self.crypto.getRandomValues(new Uint16Array(8)).constructor,
                      Uint16Array, "crypto.getRandomValues(new Uint16Array(8))")

        assert_equals(self.crypto.getRandomValues(new Int32Array(8)).constructor,
                      Int32Array, "crypto.getRandomValues(new Int32Array(8))")
        assert_equals(self.crypto.getRandomValues(new Uint32Array(8)).constructor,
                      Uint32Array, "crypto.getRandomValues(new Uint32Array(8))")
    }, "Integer arrays")

    test(function() {
        assert_throws("QuotaExceededError", function() {
            self.crypto.getRandomValues(new Int8Array(65537))
        }, "crypto.getRandomValues length over 65536")
    }, "Large length")
}
