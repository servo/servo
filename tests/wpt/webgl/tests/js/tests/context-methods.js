"use strict";

// Properties to be ignored because they were added in versions of the
// spec that are backward-compatible with this version
const IGNORED_METHODS = [
  // There is no official spec for the commit API yet, the proposal link is:
  // https://wiki.whatwg.org/wiki/OffscreenCanvas
  "commit",

  // For WebXR integration:
  "makeXRCompatible",
];

function assertFunction(v, f) {
  try {
    if (typeof v[f] != "function") {
      testFailed(`Property either does not exist or is not a function: ${f}`);
      return false;
    } else {
      return true;
    }
  } catch(e) {
    testFailed(`Trying to access the property '${f}' threw an error: ${e.toString()}`);
  }
}

function testContextMethods(gl, requiredContextMethods) {
  const acceptableMethods = [].concat(requiredContextMethods, IGNORED_METHODS);

  let passed = true;
  requiredContextMethods.forEach(method => {
    const r = assertFunction(gl, method);
    passed = passed && r;
  });
  if (passed) {
    testPassed("All WebGL methods found.");
  }
  let extended = false;
  for (let propertyName of Object.getOwnPropertyNames(gl)) {
    if (typeof gl[propertyName] == "function" && !acceptableMethods.includes(propertyName)) {
      if (!extended) {
        extended = true;
        testFailed("Also found the following extra methods:");
      }
      testFailed(propertyName);
    }
  }

  if (!extended) {
    testPassed("No extra methods found on WebGL context.");
  }
}
