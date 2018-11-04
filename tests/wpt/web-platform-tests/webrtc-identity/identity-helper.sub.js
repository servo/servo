'use strict';

/*
  In web-platform-test, a number of domains are required to be set up locally.
  The list is available at docs/_writing-tests/server-features.md. The
  appropriate hosts file entries can be generated with the WPT CLI via the
  following command: `wpt make-hosts-file`.
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
  const domainA = '{{domains[www]}}';
  const domainB = '{{domains[www1]}}';
  const domainC = '{{domains[www2]}}';

  if(window.location.hostname === domainA) {
    return [domainB, domainC];
  } else if(window.location.hostname === domainB) {
    return [domainA, domainC];
  } else {
    return [domainA, domainB];
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
