// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js
// META: script=./resources/common.js

// A set of tests, checking cookies defined from within a credentialless iframe
// continue to work.

const same_origin = get_host_info().HTTPS_ORIGIN;
const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
const cookie_key = token()

const credentialless_iframe = newIframeCredentialless(cross_origin);

// Install some helper functions in the child to observe Cookies:
promise_setup(async () => {
  await send(credentialless_iframe, `
    window.getMyCookie = () => {
      const value = "; " + document.cookie;
      const parts = value.split("; ${cookie_key}=");
      if (parts.length !== 2)
        return undefined
      return parts.pop().split(';').shift();
    };

    window.nextCookieValue = () => {
      return new Promise(resolve => {
        const old_cookie = getMyCookie();
        let timeToLive = 40; // 40 iterations of 100ms = 4s;
        const interval = setInterval(() => {
          const next_cookie_value = getMyCookie();
          timeToLive--;
          if (old_cookie !== next_cookie_value || timeToLive <= 0) {
            clearInterval(interval);
            resolve(next_cookie_value)
          }
        }, 100)
      });
    };
  `);
}, "Setup");

promise_test(async test => {
  const this_token = token();
  send(credentialless_iframe, `
    document.cookie = "${cookie_key}=cookie_value_1";
    send("${this_token}", getMyCookie());
  `);

  assert_equals(await receive(this_token), "cookie_value_1");
}, "Set/Get cookie via JS API");

promise_test(async test => {
  const resource_token = token();
  send(credentialless_iframe, `
    fetch("${showRequestHeaders(cross_origin, resource_token)}");
  `);

  const request_headers = JSON.parse(await receive(resource_token));
  const cookie_value = parseCookies(request_headers)[cookie_key];
  assert_equals(cookie_value, "cookie_value_1");
}, "Get Cookie via subresource requests");

promise_test(async test => {
  const resource_token = token();
  const resource_url = cross_origin + "/common/blank.html?pipe=" +
    `|header(Set-Cookie,${cookie_key}=cookie_value_2;Path=/common/dispatcher)`;
  const this_token = token();
  send(credentialless_iframe, `
    const next_cookie_value = nextCookieValue();
    fetch("${resource_url}");
    send("${this_token}", await next_cookie_value);
  `);

  assert_equals(await receive(this_token), "cookie_value_2");
}, "Set Cookie via subresource requests");

promise_test(async test => {
  const resource_token = token();
  const resource_url = cross_origin + "/common/blank.html?pipe=" +
    `|header(Set-Cookie,${cookie_key}=cookie_value_3;Path=/common/dispatcher)`;
  const this_token = token();
  send(credentialless_iframe, `
    const next_cookie_value = nextCookieValue();
    const iframe = document.createElement("iframe");
    iframe.src = "${resource_url}";
    document.body.appendChild(iframe);
    send("${this_token}", await next_cookie_value);
  `);

  assert_equals(await receive(this_token), "cookie_value_3");
}, "Set Cookie via navigation requests");
