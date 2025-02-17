// META: script=helper.js

//
// Validate signature-based SRI's interaction between signed script responses
// and `<script integrity>` assertions.
//

generate_script_test(kUnsignedShouldExecute, "", EXPECT_LOADED,
                     "No signature, no integrity check: loads.");

generate_script_test(kUnsignedShouldExecute, "ed25519-???", EXPECT_LOADED,
                     "No signature, malformed integrity check: loads.");

generate_script_test(kUnsignedShouldBlock, `ed25519-${kValidKeys['rfc']}`, EXPECT_BLOCKED,
                     "No signature, integrity check: blocked.");

// Should load:
generate_script_test(kSignedShouldExecute, "", EXPECT_LOADED,
                     "Valid signature, no integrity check: loads.");
generate_script_test(kSignedShouldExecute, "ed25519-???", EXPECT_LOADED,
                     "Valid signature, malformed integrity check: loads.");
generate_script_test(kSignedShouldExecute, `ed25519-${kValidKeys['rfc']}`, EXPECT_LOADED,
                     "Valid signature, valid integrity check: loads.");
generate_script_test(kSignedShouldExecute, `ed25519-${kValidKeys['rfc']} ed25519-${kValidKeys['arbitrary']}`, EXPECT_LOADED,
                     "Valid signature, one matching integrity check: loads.");

// Should block:
generate_script_test(kSignedShouldBlock, `ed25519-${kValidKeys['arbitrary']}`, EXPECT_BLOCKED,
                     "Valid signature, mismatched integrity check: blocked.");

// Executable and non-executable scripts signed with RFC's test key and the arbitrary key:
generate_script_test(kMultiplySignedShouldExecute, "", EXPECT_LOADED,
                     "Valid signatures, no integrity check: loads.");
generate_script_test(kMultiplySignedShouldExecute, "ed25519-???", EXPECT_LOADED,
                     "Valid signatures, malformed integrity check: loads.");
generate_script_test(kMultiplySignedShouldExecute, `ed25519-${kValidKeys['rfc']}`, EXPECT_LOADED,
                     "Valid signatures, integrity check matches one: loads.");
generate_script_test(kMultiplySignedShouldExecute, `ed25519-${kValidKeys['arbitrary']}`, EXPECT_LOADED,
                     "Valid signatures, integrity check matches the other: loads.");
generate_script_test(kMultiplySignedShouldExecute, `ed25519-${kValidKeys['rfc']} ed25519-${kValidKeys['arbitrary']}`, EXPECT_LOADED,
                     "Valid signatures, integrity check matches both: loads.");

// Should block:
generate_script_test(kMultiplySignedShouldBlock, `ed25519-${kInvalidKey}`, EXPECT_BLOCKED,
                     "Valid signatures, integrity check matches neither: blocked.");
