const cspDirectives = [
  // https://w3c.github.io/trusted-types/dist/spec/#require-trusted-types-for-csp-directive
  "require-trusted-types-for",
  // https://w3c.github.io/trusted-types/dist/spec/#trusted-types-csp-directive
  "trusted-types",
  // https://w3c.github.io/webappsec-csp/#script-src
  "script-src",
];

// A generic helper that runs function fn and return a promise resolving with
// an array of reported violations for trusted type directives and a possible
// exception thrown.
function trusted_type_violations_and_exception_for(fn) {
  return new Promise((resolve, reject) => {
    // Listen for security policy violations.
    let result = { violations: [], exception: null };
    let handler = e => {
      if (cspDirectives.includes(e.effectiveDirective)) {
        result.violations.push(e);
      } else if (e.effectiveDirective === "object-src") {
        document.removeEventListener("securitypolicyviolation", handler);
        e.stopPropagation();
        resolve(result);
      } else {
        reject(`Unexpected violation for directive ${e.effectiveDirective}`);
      }
    }
    document.addEventListener("securitypolicyviolation", handler);

    // Run the specified function and record any exception.
    try {
      fn();
    } catch(e) {
      result.exception = e;
    }

    // Force an "object-src" violation, to make sure all the previous violations
    // have been delivered. This assumes the test file's associated .headers
    // file contains Content-Security-Policy: object-src 'none'.
    var o = document.createElement('object');
    o.type = "video/mp4";
    o.data = "dummy.webm";
    document.body.appendChild(o);
  });
}

// Helper function when we expect one violation and exception.
async function trusted_type_violation_for(expectedException, fn) {
  let {violations, exception} =
      await trusted_type_violations_and_exception_for(fn);
  assert_equals(violations.length, 1, "a single violation reported");
  assert_true(exception instanceof expectedException, `${expectedException.prototype} exception reported`);
  return violations[0];
}

// Helper function when we expect no violation or exception.
async function no_trusted_type_violation_for(fn) {
  let {violations, exception} =
      await trusted_type_violations_and_exception_for(fn);
  assert_equals(violations.length, 0, "no violation reported");
  assert_equals(exception, null, "no exception thrown");
}

async function trusted_type_violation_without_exception_for(fn) {
  let {violations, exception} =
      await trusted_type_violations_and_exception_for(fn);
  assert_equals(violations.length, 1, "a single violation reported");
  assert_equals(exception, null, "no exception thrown");
  return violations[0];
}

function clipSampleIfNeeded(sample) {
  const clippedSampleLength = 40;

  // Clipping is a bit ambiguous when the sample contains surrogate pairs, so
  // avoid that in our tests for now.
  // https://github.com/w3c/trusted-types/issues/577
  assert_equals(sample.match(/[\uD800-\uDBFF][\uDC00-\uDFFF]/), null);

  return sample.substring(0, clippedSampleLength);
}
