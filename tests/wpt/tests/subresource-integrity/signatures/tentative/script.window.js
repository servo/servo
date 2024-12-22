// META: script=helper.js

//
// Validate signature-based SRI's interaction between signed script responses
// and `<script integrity>` assertions.
//


const kScriptToExecute = {
  body: "window.hello = `world`;",
  hash: "PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=",

  signatures: {
    // ```
    // "identity-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    // "@signature-params": ("identity-digest";sf);alg="ed25519";keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
    // ```
    rfc: "pRcIRwdXaZL4XqkAo1a7mXIlzETMgG93JWWgqDlx6XhWe8mC8umiEgbI3afULpzT1Buro4ZJfzEXwy8tC5HaCA==",

    // ```
    // "identity-digest";sf: sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:
    // "@signature-params": ("identity-digest";sf);alg="ed25519";keyid="xDnP380zcL4rJ76rXYjeHlfMyPZEOqpJYjsjEppbuXE=";tag="sri"
    // ```
    arbitrary: "6zUKqibVA3CzFvQj6a+irKnOB9ZY2ky5opG7TMpFF0BtvJ1oAjoVjW3uObPlD/PBOrmkXFNRNwv3PVerE12FDQ=="
  }
};

const kScriptToBlock = {
  body: "assert_unreached(`This code should not execute.`);",
  hash: "FUSFR1N3vTmSGbI7q9jaMbHq+ogNeBfpznOIufaIfpc=",

  signatures: {
    // ```
    // "identity-digest";sf: sha-256=:FUSFR1N3vTmSGbI7q9jaMbHq+ogNeBfpznOIufaIfpc=:
    // "@signature-params": ("identity-digest";sf);alg="ed25519";keyid="JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=";tag="sri"
    // ```
    rfc: "mXbPPr9LIwClnGOoPM/7mlRT3PfgCHnF4E5te6LocGWplqcxS6qKQoUPo/rnU8BxCY56/nI4BuGtgyjPr2lQCg==",

    // ```
    // "identity-digest";sf: sha-256=:FUSFR1N3vTmSGbI7q9jaMbHq+ogNeBfpznOIufaIfpc=:
    // "@signature-params": ("identity-digest";sf);alg="ed25519";keyid="xDnP380zcL4rJ76rXYjeHlfMyPZEOqpJYjsjEppbuXE";tag="sri"
    // ```
    arbitrary: "FGQbZOeQIqXQLbooOWExK2M756WCcT4rcszNsXX6+Z6Wdofh4GKuXoFcFSdiYiGNamFMHEW6/BRMoVVjtnGwAg=="
  }
};

//
// Equally exciting helper functions
//

// Executable: unsigned.
const kUnsigned = { body: kScriptToExecute['body'] };
generate_script_test(kUnsigned, "", EXPECT_LOADED,
                     "No signature, no integrity check: loads.");

generate_script_test(kUnsigned, "ed25519-???", EXPECT_LOADED,
                     "No signature, malformed integrity check: loads.");

generate_script_test(kUnsigned, `ed25519-${kValidKeys['rfc']}`, EXPECT_BLOCKED,
                     "No signature, valid integrity check: loads.");

// Executable and non-executable scripts signed with RFC's test key.
const kSignedShouldExecute = {
  body: kScriptToExecute['body'],
  digest: `sha-256=:${kScriptToExecute['hash']}:`,
  signatureInput: `signature=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['rfc']}";tag="sri"`,
  signature: `signature=:${kScriptToExecute['signatures']['rfc']}:`
};
const kSignedShouldBlock = {
  body: kScriptToBlock['body'],
  digest: `sha-256=:${kScriptToBlock['hash']}:`,
  signatureInput: `signature=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['rfc']}";tag="sri"`,
  signature: `signature=:${kScriptToBlock['signatures']['rfc']}:`
};

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
const kMultiplySignedShouldExecute = {
  body: kScriptToExecute['body'],
  digest: `sha-256=:${kScriptToExecute['hash']}:`,
  signatureInput: `signature1=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['rfc']}";tag="sri", ` +
                  `signature2=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['arbitrary']}";tag="sri"`,
  signature: `signature1=:${kScriptToExecute['signatures']['rfc']}:, ` +
             `signature2=:${kScriptToExecute['signatures']['arbitrary']}:`
};
const kMultiplySignedShouldBlock = {
  body: kScriptToBlock['body'],
  digest: `sha-256=:${kScriptToBlock['hash']}:`,
  signatureInput: `signature1=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['rfc']}";tag="sri", ` +
                  `signature2=("identity-digest";sf);alg="ed25519";keyid="${kValidKeys['arbitrary']}";tag="sri"`,
  signature: `signature1=:${kScriptToBlock['signatures']['rfc']}:, ` +
             `signature2=:${kScriptToBlock['signatures']['arbitrary']}:`
};
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
