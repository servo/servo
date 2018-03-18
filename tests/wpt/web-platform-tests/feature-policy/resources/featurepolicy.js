// Tests whether a feature that is enabled/disabled by feature policy works
// as expected.
// Arguments:
//    feature_description: a short string describing what feature is being
//        tested. Examples: "usb.GetDevices()", "PaymentRequest()".
//    test: test created by testharness. Examples: async_test, promise_test.
//    src: URL where a feature's availability is checked. Examples:
//        "/feature-policy/resources/feature-policy-payment.html",
//        "/feature-policy/resources/feature-policy-usb.html".
//    expect_feature_available: a callback(data, feature_description) to
//        verify if a feature is avaiable or unavailable as expected.
//        The file under the path "src" defines what "data" is sent back as a
//        pistMessage. Inside the callback, some tests (e.g., EXPECT_EQ,
//        EXPECT_TRUE, etc) are run accordingly to test a feature's
//        availability.
//        Example: expect_feature_available_default(data, feature_description).
//    feature_name: Optional argument, only provided when testing iframe allow
//      attribute. "feature_name" is the feature name of a policy controlled
//      feature (https://wicg.github.io/feature-policy/#features).
//      See examples at:
//      https://github.com/WICG/feature-policy/blob/gh-pages/features.md
//    allow_attribute: Optional argument, only used for testing fullscreen or
//      payment: either "allowfullscreen" or "allowpaymentrequest" is passed.
function test_feature_availability(
    feature_description, test, src, expect_feature_available, feature_name,
    allow_attribute) {
  let frame = document.createElement('iframe');
  frame.src = src;

  if (typeof feature_name !== 'undefined') {
    frame.allow = frame.allow.concat(";" + feature_name);
  }

  if (typeof allow_attribute !== 'undefined') {
    frame.setAttribute(allow_attribute, true);
  }

  window.addEventListener('message', test.step_func(function handler(evt) {
    if (evt.source === frame.contentWindow) {
      expect_feature_available(evt.data, feature_description);
      document.body.removeChild(frame);
      window.removeEventListener('message', handler);
      test.done();
    }
  }));

  document.body.appendChild(frame);
}

// Default helper functions to test a feature's availability:
function expect_feature_available_default(data, feature_description) {
  assert_true(data.enabled, feature_description);
}

function expect_feature_unavailable_default(data, feature_description) {
  assert_false(data.enabled, feature_description);
}

// This is the same as test_feature_availability() but instead of passing in a
// function to check the result of the message sent back from an iframe, instead
// just compares the result to an expected result passed in.
// Arguments:
//     test: test created by testharness. Examples: async_test, promise_test.
//     src: the URL to load in an iframe in which to test the feature.
//     expected_result: the expected value to compare to the data passed back
//         from the src page by postMessage.
//     allow_attribute: Optional argument, only provided when an allow
//         attribute should be specified on the iframe.
function test_feature_availability_with_post_message_result(
    test, src, expected_result, allow_attribute) {
  var test_result = function(data, feature_description) {
    assert_equals(data, expected_result);
  };
  test_feature_availability(null, test, src, test_result, allow_attribute);
}

// If this page is intended to test the named feature (according to the URL),
// tests the feature availability and posts the result back to the parent.
// Otherwise, does nothing.
function test_feature_in_iframe(feature_name, feature_promise_factory) {
  if (location.hash.includes(feature_name)) {
    feature_promise_factory().then(
        () => window.parent.postMessage('#OK', '*'),
        (e) => window.parent.postMessage('#' + e.name, '*'));
  }
}

// Returns true if the URL for this page indicates that it is embedded in an
// iframe.
function page_loaded_in_iframe() {
  return location.hash.startsWith('#iframe');
}

// Returns a same-origin (relative) URL suitable for embedding in an iframe for
// testing the availability of the feature.
function same_origin_url(feature_name) {
  // Append #iframe to the URL so we can detect the iframe'd version of the
  // page.
  return location.pathname + '#iframe#' + feature_name;
}

// Returns a cross-origin (absolute) URL suitable for embedding in an iframe for
// testing the availability of the feature.
function cross_origin_url(base_url, feature_name) {
  return base_url + same_origin_url(feature_name);
}

// This function runs all feature policy tests for a particular feature that
// has a default policy of "self". This includes testing:
// 1. Feature usage succeeds by default in the top level frame.
// 2. Feature usage succeeds by default in a same-origin iframe.
// 3. Feature usage fails by default in a cross-origin iframe.
// 4. Feature usage suceeds when an allow attribute is specified on a
//    cross-origin iframe.
//
// The same page which called this function will be loaded in the iframe in
// order to test feature usage there. When this function is called in that
// context it will simply run the feature and return a result back via
// postMessage.
//
// Arguments:
//     cross_origin: A cross-origin URL base to be used to load the page which
//         called into this function.
//     feature_name: The name of the feature as it should be specified in an
//         allow attribute.
//     error_name: If feature usage does not succeed, this is the string
//         representation of the error that will be passed in the rejected
//         promise.
//     feature_promise_factory: A function which returns a promise which tests
//         feature usage. If usage succeeds, the promise should resolve. If it
//         fails, the promise should reject with an error that can be
//         represented as a string.
function run_all_fp_tests_allow_self(
    cross_origin, feature_name, error_name, feature_promise_factory) {
  // This may be the version of the page loaded up in an iframe. If so, just
  // post the result of running the feature promise back to the parent.
  if (page_loaded_in_iframe()) {
    test_feature_in_iframe(feature_name, feature_promise_factory);
    return;
  }

  // Run the various tests.
  // 1. Allowed in top-level frame.
  promise_test(
      () => feature_promise_factory(),
      'Default "' + feature_name +
          '" feature policy ["self"] allows the top-level document.');

  // 2. Allowed in same-origin iframe.
  const same_origin_frame_pathname = same_origin_url(feature_name);
  async_test(
      t => {
        test_feature_availability_with_post_message_result(
            t, same_origin_frame_pathname, '#OK');
      },
      'Default "' + feature_name +
          '" feature policy ["self"] allows same-origin iframes.');

  // 3. Blocked in cross-origin iframe.
  const cross_origin_frame_url = cross_origin_url(cross_origin, feature_name);
  async_test(
      t => {
        test_feature_availability_with_post_message_result(
            t, cross_origin_frame_url, '#' + error_name);
      },
      'Default "' + feature_name +
          '" feature policy ["self"] disallows cross-origin iframes.');

  // 4. Allowed in cross-origin iframe with "allow" attribute.
  async_test(
      t => {
        test_feature_availability_with_post_message_result(
            t, cross_origin_frame_url, '#OK', feature_name);
      },
      'Feature policy "' + feature_name +
          '" can be enabled in cross-origin iframes using "allow" attribute.');
}

// This function runs all feature policy tests for a particular feature that
// has a default policy of "*". This includes testing:
// 1. Feature usage succeeds by default in the top level frame.
// 2. Feature usage succeeds by default in a same-origin iframe.
// 3. Feature usage succeeds by default in a cross-origin iframe.
// 4. Feature usage fails when an allow attribute is specified on a
//    cross-origin iframe with a value of "feature-name 'none'".
//
// The same page which called this function will be loaded in the iframe in
// order to test feature usage there. When this function is called in that
// context it will simply run the feature and return a result back via
// postMessage.
//
// Arguments:
//     cross_origin: A cross-origin URL base to be used to load the page which
//         called into this function.
//     feature_name: The name of the feature as it should be specified in an
//         allow attribute.
//     error_name: If feature usage does not succeed, this is the string
//         representation of the error that will be passed in the rejected
//         promise.
//     feature_promise_factory: A function which returns a promise which tests
//         feature usage. If usage succeeds, the promise should resolve. If it
//         fails, the promise should reject with an error that can be
//         represented as a string.
function run_all_fp_tests_allow_all(
    cross_origin, feature_name, error_name, feature_promise_factory) {
  // This may be the version of the page loaded up in an iframe. If so, just
  // post the result of running the feature promise back to the parent.
  if (page_loaded_in_iframe()) {
    test_feature_in_iframe(feature_name, feature_promise_factory);
    return;
  }

  // Run the various tests.
  // 1. Allowed in top-level frame.
  promise_test(
      () => feature_promise_factory(),
      'Default "' + feature_name +
          '" feature policy ["*"] allows the top-level document.');

  // 2. Allowed in same-origin iframe.
  const same_origin_frame_pathname = same_origin_url(feature_name);
  async_test(
      t => {
        test_feature_availability_with_post_message_result(
            t, same_origin_frame_pathname, '#OK');
      },
      'Default "' + feature_name +
          '" feature policy ["*"] allows same-origin iframes.');

  // 3. Allowed in cross-origin iframe.
  const cross_origin_frame_url = cross_origin_url(cross_origin, feature_name);
  async_test(
      t => {
        test_feature_availability_with_post_message_result(
            t, cross_origin_frame_url, '#OK');
      },
      'Default "' + feature_name +
          '" feature policy ["*"] allows cross-origin iframes.');

  // 4. Blocked in cross-origin iframe with "allow" attribute set to 'none'.
  async_test(
      t => {
        test_feature_availability_with_post_message_result(
            t, cross_origin_frame_url, '#' + error_name,
            feature_name + " 'none'");
      },
      'Feature policy "' + feature_name +
          '" can be disabled in cross-origin iframes using "allow" attribute.');
}

// This function tests that a given policy allows each feature for the correct
// list of origins specified by the |expected_policy|.
// Arguments:
//     expected_policy: A list of {feature, allowlist} pairs where the feature is
//         enabled for every origin in the allowlist, in the |policy|.
//     policy: Either a document.policy or a iframe.policy to be tested.
//     message: A short description of what policy is being tested.
function test_allowlists(expected_policy, policy, message) {
  for (var allowlist of allowlists) {
    test(function() {
      assert_array_equals(
        policy.getAllowlistForFeature(allowlist.feature),
        allowlist.allowlist);
    }, message + ' for feature ' + allowlist.feature);
  }
}

// This function tests that a subframe's document policy allows a given feature.
// A feature is allowed in a frame either through inherited policy or specified
// by iframe allow attribute.
// Arguments:
//     test: test created by testharness. Examples: async_test, promise_test.
//     feature: feature name that should be allowed in the frame.
//     src: the URL to load in the frame.
//     allow: the allow attribute (container policy) of the iframe
function test_allowed_feature_for_subframe(message, feature, src, allow) {
  let frame = document.createElement('iframe');
  if (typeof allow !== 'undefined') {
    frame.allow = allow;
  }
  promise_test(function() {
    frame.src = src;
    return new Promise(function(resolve, reject) {
      window.addEventListener('message', function handler(evt) {
        resolve(evt.data);
      }, { once: true });
      document.body.appendChild(frame);
    }).then(function(data) {
      assert_true(data.includes(feature), feature);
    });
  }, message);
}

// This function tests that a subframe's document policy disallows a given
// feature. A feature is allowed in a frame either through inherited policy or
// specified by iframe allow attribute.
// Arguments:
//     test: test created by testharness. Examples: async_test, promise_test.
//     feature: feature name that should not be allowed in the frame.
//     src: the URL to load in the frame.
//     allow: the allow attribute (container policy) of the iframe
function test_disallowed_feature_for_subframe(message, feature, src, allow) {
  let frame = document.createElement('iframe');
  if (typeof allow !== 'undefined') {
    frame.allow = allow;
  }
  promise_test(function() {
    frame.src = src;
    return new Promise(function(resolve, reject) {
      window.addEventListener('message', function handler(evt) {
        resolve(evt.data);
      }, { once: true });
      document.body.appendChild(frame);
    }).then(function(data) {
      assert_false(data.includes(feature), feature);
    });
  }, message);
}

// This function tests that a subframe with header policy defined on a given
// feature allows and disallows the feature as expected.
// Arguments:
//     feature: feature name.
//     frame_header_policy: either *, 'self' or 'none', defines the frame
//                          document's header policy on |feature|.
//     src: the URL to load in the frame.
//     test_expects: contains 6 expected results of either |feature| is allowed
//                   or not inside of a local or remote iframe nested inside
//                   the subframe given the header policy to be either *,
//                   'slef', or 'none'.
//     test_name: name of the test.
function test_subframe_header_policy(
    feature, frame_header_policy, src, test_expects, test_name) {
  let frame = document.createElement('iframe');
  promise_test(function() {
    frame.src = src + '?pipe=sub|header(Feature-Policy,' + feature + ' '
        + frame_header_policy + ';)';
    return new Promise(function(resolve, reject) {
      let results = [];
      window.addEventListener('message', function handler(evt) {
        results.push(evt.data);
        if (results.length >= 6) {
          resolve(results);
        }
      });
      document.body.appendChild(frame);
    }).then(function(results) {
      for (var j = 0; j < results.length; j++) {
        var data = results[j];

        function test_result(message, test_expect) {
          if (test_expect) {
            assert_true(data.allowedfeatures.includes(feature), message);
          } else {
            assert_false(data.allowedfeatures.includes(feature), message);
          }
        }

        if (data.frame === 'local') {
          if (data.policy === '*') {
            test_result('local_all:', test_expects.local_all);
          }
          if (data.policy === '\'self\'') {
            test_result('local_self:', test_expects.local_self);
          }
          if (data.policy === '\'none\'') {
            test_result('local_none:', test_expects.local_none);
          }
        }

        if (data.frame === 'remote') {
          if (data.policy === '*') {
            test_result('remote_all:', test_expects.remote_all);
          }
          if (data.policy === '\'self\'') {
            test_result('remote_self:', test_expects.remote_self);
          }
          if (data.policy === '\'none\'') {
            test_result('remote_none:', test_expects.remote_none);
          }
        }
      }
    });
  }, test_name);
}

// This function tests that frame policy allows a given feature correctly. A
// feature is allowed in a frame either through inherited policy or specified
// by iframe allow attribute.
// Arguments:
//     feature: feature name.
//     src: the URL to load in the frame.
//     test_expect: boolean value of whether the feature should be allowed.
//     allow: optional, the allow attribute (container policy) of the iframe.
//     allowfullscreen: optional, boolean value of allowfullscreen attribute.
function test_frame_policy(
    feature, src, test_expect, allow, allowfullscreen) {
  let frame = document.createElement('iframe');
  document.body.appendChild(frame);
  // frame_policy should be dynamically updated as allow and allowfullscreen is
  // updated.
  var frame_policy = frame.policy;
  if (typeof allow !== 'undefined') {
    frame.setAttribute('allow', allow);
  }
  if (!!allowfullscreen) {
    frame.setAttribute('allowfullscreen', true);
  }
  frame.src = src;
  if (test_expect) {
    assert_true(frame_policy.allowedFeatures().includes(feature));
  } else {
    assert_false(frame_policy.allowedFeatures().includes(feature));
  }
}
