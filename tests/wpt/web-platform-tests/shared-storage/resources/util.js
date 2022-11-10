'use strict';

// Execute all shared storage methods and capture their errors. Return true if
// the permissions policy allows all of them; return false if the permissions
// policy disallows all of them. Precondition: only these two outcomes are
// possible.
async function AreSharedStorageMethodsAllowedByPermissionsPolicy() {
  let permissionsPolicyDeniedCount = 0;
  const errorMessage = 'The \"shared-storage\" Permissions Policy denied the method on window.sharedStorage.';

  try {
    await window.sharedStorage.worklet.addModule('/shared-storage/resources/simple-module.js');
  } catch (e) {
    assert_equals(e.message, errorMessage);
    ++permissionsPolicyDeniedCount;
  }

  try {
    await window.sharedStorage.run('operation');
  } catch (e) {
    assert_equals(e.message, errorMessage);
    ++permissionsPolicyDeniedCount;
  }

  try {
    // Run selectURL() with without addModule() and this should always fail.
    // Check the error message to distinguish between the permissions policy
    // error and the missing addModule() error.
    await sharedStorage.selectURL("operation", [{url: "1.html"}]);
    assert_unreached("did not fail");
  } catch (e) {
    if (e.message === errorMessage) {
      ++permissionsPolicyDeniedCount;
    }
  }

  try {
    await window.sharedStorage.set('a', 'b');
  } catch (e) {
    assert_equals(e.message, errorMessage);
    ++permissionsPolicyDeniedCount;
  }

  try {
    await window.sharedStorage.append('a', 'b');
  } catch (e) {
    assert_equals(e.message, errorMessage);
    ++permissionsPolicyDeniedCount;
  }

  try {
    await window.sharedStorage.clear();
  } catch (e) {
    assert_equals(e.message, errorMessage);
    ++permissionsPolicyDeniedCount;
  }

  try {
    await window.sharedStorage.delete('a');
  } catch (e) {
    assert_equals(e.message, errorMessage);
    ++permissionsPolicyDeniedCount;
  }

  if (permissionsPolicyDeniedCount === 0)
    return true;

  return false;
}