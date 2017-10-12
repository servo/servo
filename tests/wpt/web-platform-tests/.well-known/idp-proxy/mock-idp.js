'use strict';

// Code is based on the following editor draft:
//   https://w3c.github.io/webrtc-pc/archives/20170605/webrtc.html

/*
  mock-idp.js is a naive IdP that provides absolutely no
  security for authentication. It can generate identity
  assertion for whatever identity that is requested.

  mock-idp.js validates identity assertion by simply decoding
  the JSON and return whatever that is inside, with no integrity
  protection and thus can be spoofed by anyone.

  While being not practical at all, mock-idp.js allows us
  to test various aspects of the identity API and allow tests
  to manipulate the IdP at will.
 */

// We pass around test options as query string to instruct
// the test IdP proxy script on what actions to perform.
// This hack is based on the fact that query string is allowed
// when specifying the IdP protocol.
function parseQueryString(urlStr) {
  const url = new URL(urlStr);
  const result = {};
  for(const [key, value] of url.searchParams) {
    result[key] = value;
  }
  return result;
}

/*
  9.2.1.  Interface Exposed by Identity Providers
    callback GenerateAssertionCallback =
      Promise<RTCIdentityAssertionResult> (
        DOMString contents,
        DOMString origin,
        RTCIdentityProviderOptions options);

    dictionary RTCIdentityProviderOptions {
      DOMString protocol = "default";
      DOMString usernameHint;
      DOMString peerIdentity;
    };

    dictionary RTCIdentityAssertionResult {
      required RTCIdentityProviderDetails idp;
      required DOMString                  assertion;
    };

    dictionary RTCIdentityProviderDetails {
      required DOMString domain;
               DOMString protocol = "default";
    };
 */

// In RTCIdentityProviderGlobalScope, global is self
const global = self;
const query = parseQueryString(global.location);

// Generate a naive identity assertion. The result assertion
// is a JSON string that report the various parameters
// received by this function.
//   watermark - a special mark to make sure the result is returned
//               from this function
//   args - the function arguments received
//   env - some global variable values when this function is called
//   query - the parsed query string of the script URL
function generateAssertion(contents, origin, options) {
  const args = {
    contents, origin, options
  };

  const env = {
    origin: global.origin,
    location: global.location
  };

  const assertion = {
    watermark: 'mock-idp.js.watermark',
    args,
    env,
    query
  };

  const idp = {
    domain: global.location.host,
    protocol: 'mock-idp.js'
  };

  const assertionStr = JSON.stringify(assertion);

  const { generatorAction } = query;

  if(generatorAction === 'throw-error') {
    const err = new Error('Mock Internal IdP Error');
    err.idpErrorInfo = query.errorInfo;
    throw err;

  } else if(generatorAction === 'require-login') {
    const err = new RTCError('idp-need-login');
    err.idpLoginUrl = `${self.origin}/login`;
    err.idpErrorInfo = 'login required';
    throw err;

  } else if(generatorAction === 'return-custom-idp') {
    const { domain, protocol } = query;

    return {
      idp: {
        domain,
        protocol
      },
      assertion: assertionStr
    };

  } else if(generatorAction === 'return-invalid-result') {
    return 'invalid-result';

  } else {
    return {
      idp,
      assertion: assertionStr
    };
  }
}

/*
  9.2.1.  Interface Exposed by Identity Providers
    callback ValidateAssertionCallback =
      Promise<RTCIdentityValidationResult> (
        DOMString assertion,
        DOMString origin);

    dictionary RTCIdentityValidationResult {
      required DOMString identity;
      required DOMString contents;
    };
 */
function validateAssertion(assertionStr, origin) {
  const assertion = JSON.parse(assertionStr);

  const { param, query } = assertion;
  const { contents, options } = param;

  const identity = options.usernameHint;

  const {
    validatorAction
  } = query;

  if(validatorAction === 'throw-error') {
    const err = new Error('Mock Internal IdP Error');
    err.idpErrorInfo = query.errorInfo;
    throw err;

  } else if(validatorAction === 'return-custom-contents') {
    const { contents } = query;
    return {
      identity,
      contents
    };

  } else {
    return {
      identity, contents
    };
  }
}

/*
  9.2.  Registering an IdP Proxy
    [Global,
     Exposed=RTCIdentityProviderGlobalScope]
    interface RTCIdentityProviderGlobalScope : WorkerGlobalScope {
      readonly attribute RTCIdentityProviderRegistrar rtcIdentityProvider;
    };

    [Exposed=RTCIdentityProviderGlobalScope]
    interface RTCIdentityProviderRegistrar {
      void register(RTCIdentityProvider idp);
    };

    dictionary RTCIdentityProvider {
      required GenerateAssertionCallback generateAssertion;
      required ValidateAssertionCallback validateAssertion;
    };
 */

// if global.rtcIdentityProvider is defined, and the caller do not ask
// to not register through query string, register our assertion callbacks.
if(global.rtcIdentityProvider && query.action !== 'do-not-register') {
  global.rtcIdentityProvider.register({
    generateAssertion,
    validateAssertion
  });
}
