// Copyright (C) 2025 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Verify that ImportDeclaration can be correctly parsed.
esid: sec-modules
features: [source-phase-imports]
flags: [async]
includes: [asyncHelpers.js]
---*/

function assertImportSourceResolutionFailure(specifier) {
  // Import the module and assert that the promise is rejected with a host
  // defined error during the resolution phase.
  // Note that this is not a `import.source`.
  return import(specifier).then(
    () => {
      throw new Test262Error(`${specifier}: Promise should be rejected`);
    },
    error => {
      if (error instanceof SyntaxError) {
        throw new Test262Error(`${specifier}: Promise should be rejected with a non-SyntaxError`);
      }
    }
  );
}

asyncTest(async function () {
  await assertImportSourceResolutionFailure('./import-source-binding-name_FIXTURE.js');
  await assertImportSourceResolutionFailure('./import-source-binding-name-2_FIXTURE.js');
  await assertImportSourceResolutionFailure('./import-source-newlines_FIXTURE.js');
});
