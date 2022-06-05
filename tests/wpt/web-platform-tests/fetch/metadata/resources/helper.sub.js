'use strict';

/**
 * Construct a URL which, when followed, will trigger redirection through zero
 * or more specified origins and ultimately resolve in the Python handler
 * `record-headers.py`.
 *
 * @param {string} key - the WPT server "stash" name where the request's
 *                       headers should be stored
 * @param {string[]} [origins] - zero or more origin names through which the
 *                               request should pass; see the function
 *                               implementation for a completel list of names
 *                               and corresponding origins; If specified, the
 *                               final origin will be used to access the
 *                               `record-headers.py` hander.
 * @param {object} [params] - a collection of key-value pairs to include as
 *                            URL "search" parameters in the final request to
 *                            `record-headers.py`
 *
 * @returns {string} an absolute URL
 */
function makeRequestURL(key, origins, params) {
    const byName = {
        httpOrigin: 'http://{{host}}:{{ports[http][0]}}',
        httpSameSite: 'http://{{hosts[][www]}}:{{ports[http][0]}}',
        httpCrossSite: 'http://{{hosts[alt][]}}:{{ports[http][0]}}',
        httpsOrigin: 'https://{{host}}:{{ports[https][0]}}',
        httpsSameSite: 'https://{{hosts[][www]}}:{{ports[https][0]}}',
        httpsCrossSite: 'https://{{hosts[alt][]}}:{{ports[https][0]}}'
    };
    const redirectPath = '/fetch/api/resources/redirect.py?location=';
    const path = '/fetch/metadata/resources/record-headers.py?key=' + key;

    let requestUrl = path;
    if (params) {
      requestUrl += '&' + new URLSearchParams(params).toString();
    }

    if (origins && origins.length) {
      requestUrl = byName[origins.pop()] + requestUrl;

      while (origins.length) {
        requestUrl = byName[origins.pop()] + redirectPath +
          encodeURIComponent(requestUrl);
      }
    } else {
      requestUrl = byName.httpsOrigin + requestUrl;
    }

    return requestUrl;
}

function retrieve(key, options) {
  return fetch('/fetch/metadata/resources/record-headers.py?retrieve&key=' + key)
    .then((response) => {
      if (response.status === 204 && options && options.poll) {
        return new Promise((resolve) => setTimeout(resolve, 300))
          .then(() => retrieve(key, options));
      }

      if (response.status !== 200) {
        throw new Error('Failed to query for recorded headers.');
      }

      return response.text().then((text) => JSON.parse(text));
    });
}
