// Set up global variables.
(_ => {
  var HOST = '{{host}}';
  var CROSS_ORIGIN_HOST = '{{hosts[alt][]}}';
  var WSS_PORT = ':{{ports[wss][0]}}';
  var HTTPS_PORT = ':{{ports[https][0]}}';

  window.WSS_ORIGIN = 'wss://' + HOST + WSS_PORT;
  window.WSS_CROSS_SITE_ORIGIN = 'wss://' + CROSS_ORIGIN_HOST + WSS_PORT;
  window.HTTPS_ORIGIN = 'https://' + HOST + HTTPS_PORT;
  window.HTTPS_CROSS_SITE_ORIGIN = 'https://' + CROSS_ORIGIN_HOST + HTTPS_PORT;
})();

// Sets a cookie with each SameSite option.
function setSameSiteCookies(origin, value) {
  return new Promise(resolve => {
    const ws = new WebSocket(origin + '/set-cookies-samesite?value=' + value);
    ws.onopen = () => {
      ws.close();
    };
    ws.onclose = resolve;
  });
}

// Clears cookies set by setSameSiteCookies().
function clearSameSiteCookies(origin) {
  return new Promise(resolve => {
    const ws = new WebSocket(origin + '/set-cookies-samesite?clear');
    ws.onopen = () => ws.close();
    ws.onclose = resolve;
  });
}

// Gets value of Cookie header sent in request.
function connectAndGetRequestCookiesFrom(origin) {
  return new Promise((resolve, reject) => {
      var ws = new WebSocket(origin + '/echo-cookie');
      ws.onmessage = evt => {
          var cookies = evt.data
          resolve(cookies);
          ws.onerror = undefined;
          ws.onclose = undefined;
      };
      ws.onerror = () => reject('Unexpected error event');
      ws.onclose = evt => reject('Unexpected close event: ' + JSON.stringify(evt));
  });
}

// Assert that a given cookie is or is not present in the string |cookies|.
function assertCookie(cookies, name, value, present) {
  var assertion = present ? assert_true : assert_false;
  var description = name + '=' + value + ' cookie is' +
                    (present ? ' ' : ' not ') + 'present.';
  var re = new RegExp('(?:^|; )' + name + '=' + value + '(?:$|;)');
  assertion(re.test(cookies), description);
}

