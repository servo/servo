'use strict';

/*
  In web-platform-test, the following domains are required to be set up locally:
    127.0.0.1   web-platform.test
    127.0.0.1   www.web-platform.test
    127.0.0.1   www1.web-platform.test
    127.0.0.1   www2.web-platform.test
    127.0.0.1   xn--n8j6ds53lwwkrqhv28a.web-platform.test
    127.0.0.1   xn--lve-6lad.web-platform.test
    0.0.0.0     nonexistent-origin.web-platform.test
 */

/*
    dictionary RTCIdentityProviderDetails {
      required DOMString domain;
               DOMString protocol = "default";
    };
 */

// Parse a base64 JSON encoded string returned from getIdentityAssertion().
// This is also the string that is set in the a=identity line.
// Returns a { idp, assertion } where idp is of type RTCIdentityProviderDetails
// and assertion is the deserialized JSON that was returned by the
// IdP proxy's generateAssertion() function.
function parseAssertionResult(assertionResultStr) {
  const assertionResult = JSON.parse(atob(assertionResultStr));

  const { idp } = assertionResult;
  const assertion = JSON.parse(assertionResult.assertion);

  return { idp, assertion };
}

// Return two distinct IdP domains that are different from current domain
function getIdpDomains() {
  if(window.location.hostname === 'www1.web-platform.test') {
    return ['www.web-platform.test', 'www2.web-platform.test'];
  } else if(window.location.hostname === 'www2.web-platform.test') {
    return ['www.web-platform.test', 'www1.web-platform.test'];
  } else {
    return ['www1.web-platform.test', 'www2.web-platform.test'];
  }
}

function assert_rtcerror_rejection(errorDetail, promise, desc) {
  return promise.then(
    res => {
      assert_unreached(`Expect promise to be rejected with RTCError, but instead got ${res}`);
    }, err => {
      assert_true(err instanceof RTCError,
        'Expect error object to be instance of RTCError');

      assert_equals(err.errorDetail, errorDetail,
        `Expect RTCError object have errorDetail set to ${errorDetail}`);

      return err;
    });
}

// construct a host string consist of domain and optionally port
// If the default HTTP/HTTPS port is used, window.location.port returns
// empty string.
function hostString(domain, port) {
  if(port === '') {
    return domain;
  } else {
    return `${domain}:${port}`;
  }
}
