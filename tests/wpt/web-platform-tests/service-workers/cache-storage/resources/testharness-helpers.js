/*
 * testharness-helpers contains various useful extensions to testharness.js to
 * allow them to be used across multiple tests before they have been
 * upstreamed. This file is intended to be usable from both document and worker
 * environments, so code should for example not rely on the DOM.
 */

// Returns a promise that fulfills after the provided |promise| is fulfilled.
// The |test| succeeds only if |promise| rejects with an exception matching
// |code|. Accepted values for |code| follow those accepted for assert_throws().
// The optional |description| describes the test being performed.
//
// E.g.:
//   assert_promise_rejects(
//       new Promise(...), // something that should throw an exception.
//       'NotFoundError',
//       'Should throw NotFoundError.');
//
//   assert_promise_rejects(
//       new Promise(...),
//       new TypeError(),
//       'Should throw TypeError');
function assert_promise_rejects(promise, code, description) {
  return promise.then(
    function() {
      throw 'assert_promise_rejects: ' + description + ' Promise did not reject.';
    },
    function(e) {
      if (code !== undefined) {
        assert_throws(code, function() { throw e; }, description);
      }
    });
}

// Helper for testing with Headers objects. Compares Headers instances
// by serializing |expected| and |actual| to arrays and comparing.
function assert_header_equals(actual, expected, description) {
    assert_class_string(actual, "Headers", description);
    var header, actual_headers = [], expected_headers = [];
    for (header of actual)
        actual_headers.push(header[0] + ": " + header[1]);
    for (header of expected)
        expected_headers.push(header[0] + ": " + header[1]);
    assert_array_equals(actual_headers, expected_headers,
                        description + " Headers differ.");
}

// Helper for testing with Response objects. Compares simple
// attributes defined on the interfaces, as well as the headers. It
// does not compare the response bodies.
function assert_response_equals(actual, expected, description) {
    assert_class_string(actual, "Response", description);
    ["type", "url", "status", "ok", "statusText"].forEach(function(attribute) {
        assert_equals(actual[attribute], expected[attribute],
                      description + " Attributes differ: " + attribute + ".");
    });
    assert_header_equals(actual.headers, expected.headers, description);
}

// Assert that the two arrays |actual| and |expected| contain the same
// set of Responses as determined by assert_response_equals. The order
// is not significant.
//
// |expected| is assumed to not contain any duplicates.
function assert_response_array_equivalent(actual, expected, description) {
    assert_true(Array.isArray(actual), description);
    assert_equals(actual.length, expected.length, description);
    expected.forEach(function(expected_element) {
        // assert_response_in_array treats the first argument as being
        // 'actual', and the second as being 'expected array'. We are
        // switching them around because we want to be resilient
        // against the |actual| array containing duplicates.
        assert_response_in_array(expected_element, actual, description);
    });
}

// Asserts that two arrays |actual| and |expected| contain the same
// set of Responses as determined by assert_response_equals(). The
// corresponding elements must occupy corresponding indices in their
// respective arrays.
function assert_response_array_equals(actual, expected, description) {
    assert_true(Array.isArray(actual), description);
    assert_equals(actual.length, expected.length, description);
    actual.forEach(function(value, index) {
        assert_response_equals(value, expected[index],
                               description + " : object[" + index + "]");
    });
}

// Equivalent to assert_in_array, but uses assert_response_equals.
function assert_response_in_array(actual, expected_array, description) {
    assert_true(expected_array.some(function(element) {
        try {
            assert_response_equals(actual, element);
            return true;
        } catch (e) {
            return false;
        }
    }), description);
}
