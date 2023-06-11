// Utility functions to help testing keepalive requests.

// Returns a URL to an iframe that loads a keepalive URL on iframe loaded.
//
// The keepalive URL points to a target that stores `token`. The token will then
// be posted back on iframe loaded to the parent document.
// `method` defaults to GET.
// `frameOrigin` to specify the origin of the iframe to load. If not set,
// default to a different site origin.
// `requestOrigin` to specify the origin of the fetch request target.
// `sendOn` to specify the name of the event when the keepalive request should
// be sent instead of the default 'load'.
// `mode` to specify the fetch request's CORS mode.
// `disallowOrigin` to ask the iframe to set up a server that forbids CORS
// requests.
function getKeepAliveIframeUrl(token, method, {
  frameOrigin = 'DEFAULT',
  requestOrigin = '',
  sendOn = 'load',
  mode = 'cors',
  disallowOrigin = false
} = {}) {
  const https = location.protocol.startsWith('https');
  frameOrigin = frameOrigin === 'DEFAULT' ?
      get_host_info()[https ? 'HTTPS_NOTSAMESITE_ORIGIN' : 'HTTP_NOTSAMESITE_ORIGIN'] :
      frameOrigin;
  return `${frameOrigin}/fetch/api/resources/keepalive-iframe.html?` +
      `token=${token}&` +
      `method=${method}&` +
      `sendOn=${sendOn}&` +
      `mode=${mode}&` + (disallowOrigin ? `disallowOrigin=1&` : ``) +
      `origin=${requestOrigin}`;
}

// Returns a different-site URL to an iframe that loads a keepalive URL.
//
// By default, the keepalive URL points to a target that redirects to another
// same-origin destination storing `token`. The token will then be posted back
// to parent document.
//
// The URL redirects can be customized from `origin1` to `origin2` if provided.
// Sets `withPreflight` to true to get URL enabling preflight.
function getKeepAliveAndRedirectIframeUrl(
    token, origin1, origin2, withPreflight) {
  const https = location.protocol.startsWith('https');
  const frameOrigin =
      get_host_info()[https ? 'HTTPS_NOTSAMESITE_ORIGIN' : 'HTTP_NOTSAMESITE_ORIGIN'];
  return `${frameOrigin}/fetch/api/resources/keepalive-redirect-iframe.html?` +
      `token=${token}&` +
      `origin1=${origin1}&` +
      `origin2=${origin2}&` + (withPreflight ? `with-headers` : ``);
}

async function iframeLoaded(iframe) {
  return new Promise((resolve) => iframe.addEventListener('load', resolve));
}

// Obtains the token from the message posted by iframe after loading
// `getKeepAliveAndRedirectIframeUrl()`.
async function getTokenFromMessage() {
  return new Promise((resolve) => {
    window.addEventListener('message', (event) => {
      resolve(event.data);
    }, {once: true});
  });
}

// Tells if `token` has been stored in the server.
async function queryToken(token) {
  const response = await fetch(`../resources/stash-take.py?key=${token}`);
  const json = await response.json();
  return json;
}

// In order to parallelize the work, we are going to have an async_test
// for the rest of the work. Note that we want the serialized behavior
// for the steps so far, so we don't want to make the entire test case
// an async_test.
function assertStashedTokenAsync(testName, token, {shouldPass = true} = {}) {
  async_test((test) => {
    new Promise((resolve) => test.step_timeout(resolve, 3000))
        .then(() => {
          return queryToken(token);
        })
        .then((result) => {
          assert_equals(result, 'on');
        })
        .then(() => {
          test.done();
        })
        .catch(test.step_func((e) => {
          if (shouldPass) {
            assert_unreached(e);
          } else {
            test.done();
          }
        }));
  }, testName);
}
