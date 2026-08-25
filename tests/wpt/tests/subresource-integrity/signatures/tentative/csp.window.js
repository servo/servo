// META: script=helper.js

// Here, we're replicating many of the tests from `script.window.js`, but
// doing so in the presence of a CSP that requires the RFC's test key to
// be asserted as integrity metadata.

// First, enforce CSP:
const el = document.createElement('meta');
el.httpEquiv = "content-security-policy";
el.content = `script-src 'ed25519-${kValidKeys['rfc']}'`;
document.head.appendChild(el);

// Unsigned scripts should not load, regardless of integrity metadata:
generate_script_test(kUnsignedShouldBlock, "", EXPECT_BLOCKED,
                     "No signature, no integrity check: blocked.");

generate_script_test(kUnsignedShouldBlock, "ed25519-???", EXPECT_BLOCKED,
                     "No signature, malformed integrity check: blocked.");

generate_script_test(kUnsignedShouldBlock, `ed25519-${kValidKeys['rfc']}`, EXPECT_BLOCKED,
                     "No signature, integrity check: blocked.");

// Signed scripts should load iff valid integrity metadata is explicitly asserted:
generate_script_test(kSignedShouldBlock, "", EXPECT_BLOCKED,
                     "Valid signature, no integrity check: blocked.");
generate_script_test(kSignedShouldBlock, "ed25519-???", EXPECT_BLOCKED,
                     "Valid signature, malformed integrity check: blocked.");
generate_script_test(kSignedShouldExecute, `ed25519-${kValidKeys['rfc']}`, EXPECT_LOADED,
                     "Valid signature, valid integrity check: loads.");
generate_script_test(kSignedShouldExecute, `ed25519-${kValidKeys['rfc']} ed25519-${kValidKeys['arbitrary']}`, EXPECT_BLOCKED,
                     "Valid signature, one matching and one mismatched integrity check: blocked.");
generate_script_test(kSignedShouldBlock, `ed25519-${kValidKeys['arbitrary']}`, EXPECT_BLOCKED,
                     "Valid signature, mismatched integrity check: blocked.");

// Likewise, scripts signed with multiple signatures will still require valid integrity metadata to be asserted:
generate_script_test(kMultiplySignedShouldBlock, "", EXPECT_BLOCKED,
                     "Valid signatures, no integrity check: blocked.");
generate_script_test(kMultiplySignedShouldBlock, "ed25519-???", EXPECT_BLOCKED,
                     "Valid signatures, malformed integrity check: blocked.");
generate_script_test(kMultiplySignedShouldExecute, `ed25519-${kValidKeys['rfc']}`, EXPECT_LOADED,
                     "Valid signatures, integrity check matches one: loads.");
generate_script_test(kMultiplySignedShouldBlock, `ed25519-${kValidKeys['arbitrary']}`, EXPECT_BLOCKED,
                     "Valid signatures, integrity check matches the other: blocked.");
generate_script_test(kMultiplySignedShouldBlock, `ed25519-${kValidKeys['rfc']} ed25519-${kValidKeys['arbitrary']}`, EXPECT_BLOCKED,
                     "Valid signatures, integrity check matches both, but only one in CSP: blocked.");
generate_script_test(kMultiplySignedShouldBlock, `ed25519-${kInvalidKey}`, EXPECT_BLOCKED,
                     "Valid signatures, integrity check matches neither: blocked.");
