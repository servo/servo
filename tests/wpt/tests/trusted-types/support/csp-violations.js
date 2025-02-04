const cspDirectives = [
  // https://w3c.github.io/trusted-types/dist/spec/#require-trusted-types-for-csp-directive
  "require-trusted-types-for",
  // https://w3c.github.io/trusted-types/dist/spec/#trusted-types-csp-directive
  "trusted-types",
  // https://w3c.github.io/webappsec-csp/#script-src
  "script-src",
];

// A generic helper that runs function fn and returns a promise resolving with
// an array of reported violations and a possible exception thrown. This forces
// a "connect-src" violation before and after calling fn, to make sure we are
// not gathering violations reported before fn, and that all the violations
// reported by fn have been delivered. This assumes that the test file contains
// the CSP directive connect-src 'none' and that fn is not itself triggering
// a "connect-src" violation report.
function trusted_type_violations_and_exception_for(fn) {
  return new Promise((resolve, reject) => {
    // Listen for security policy violations.
    let result = { violations: [], exception: null };
    let handler = e => {
      if (cspDirectives.includes(e.effectiveDirective)) {
        result.violations.push(e);
      } else if (e.effectiveDirective === "connect-src") {
        self.removeEventListener("securitypolicyviolation", handler);
        e.stopPropagation();
        resolve(result);
      } else {
        reject(`Unexpected violation for directive ${e.effectiveDirective}`);
      }
    }
    self.addEventListener("securitypolicyviolation", handler);

    // Run the specified function and record any exception.
    try {
      fn();
    } catch(e) {
      result.exception = e;
    }
    // Force a connect-src violation. WebKit additionally throws a SecurityError
    // so ignore that. See https://bugs.webkit.org/show_bug.cgi?id=286744
    try {
      new EventSource("/common/blank.html");
    } catch(e) {
      if (!e instanceof DOMException || e.name !== "SecurityError") {
        throw e;
      }
    }
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
