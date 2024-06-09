/**
 * Helper functions for attribution reporting API tests.
 */

const blankURL = (base = location.origin) => new URL('/attribution-reporting/resources/reporting_origin.py', base);

const attribution_reporting_promise_test = (f, name) =>
    promise_test(async t => {
      await resetWptServer();
      return f(t);
    }, name);

const resetWptServer = () =>
    Promise
        .all([
          resetAttributionReports(eventLevelReportsUrl),
          resetAttributionReports(aggregatableReportsUrl),
          resetAttributionReports(eventLevelDebugReportsUrl),
          resetAttributionReports(attributionSuccessDebugAggregatableReportsUrl),
          resetAttributionReports(verboseDebugReportsUrl),
          resetAttributionReports(aggregatableDebugReportsUrl),
          resetRegisteredSources(),
        ]);

const eventLevelReportsUrl =
    '/.well-known/attribution-reporting/report-event-attribution';
const eventLevelDebugReportsUrl =
    '/.well-known/attribution-reporting/debug/report-event-attribution';
const aggregatableReportsUrl =
    '/.well-known/attribution-reporting/report-aggregate-attribution';
const attributionSuccessDebugAggregatableReportsUrl =
    '/.well-known/attribution-reporting/debug/report-aggregate-attribution';
const verboseDebugReportsUrl =
    '/.well-known/attribution-reporting/debug/verbose';
const aggregatableDebugReportsUrl =
    '/.well-known/attribution-reporting/debug/report-aggregate-debug';

const attributionDebugCookie = 'ar_debug=1;Secure;HttpOnly;SameSite=None;Path=/';

const pipeHeaderPattern = /[,)]/g;

// , and ) in pipe values must be escaped with \
const encodeForPipe = urlString => urlString.replace(pipeHeaderPattern, '\\$&');

const blankURLWithHeaders = (headers, origin, status) => {
  const url = blankURL(origin);

  const parts = headers.map(h => `header(${h.name},${encodeForPipe(h.value)})`);

  if (status !== undefined) {
    parts.push(`status(${encodeForPipe(status)})`);
  }

  if (parts.length > 0) {
    url.searchParams.set('pipe', parts.join('|'));
  }

  return url;
};

/**
 * Clears the source registration stash.
 */
const resetRegisteredSources = () => {
  return fetch(`${blankURL()}?clear-stash=true`);
}

/**
 * Method to clear the stash. Takes the URL as parameter. This could be for
 * event-level or aggregatable reports.
 */
const resetAttributionReports = url => {
  // The view of the stash is path-specific (https://web-platform-tests.org/tools/wptserve/docs/stash.html),
  // therefore the origin doesn't need to be specified.
  url = `${url}?clear_stash=true`;
  const options = {
    method: 'POST',
  };
  return fetch(url, options);
};

const redirectReportsTo = origin => {
  return Promise.all([
      fetch(`${eventLevelReportsUrl}?redirect_to=${origin}`, {method: 'POST'}),
      fetch(`${aggregatableReportsUrl}?redirect_to=${origin}`, {method: 'POST'})
    ]);
};

const getFetchParams = (origin, cookie) => {
  let credentials;
  const headers = [];

  if (!origin || origin === location.origin) {
    return {credentials, headers};
  }

  // https://fetch.spec.whatwg.org/#http-cors-protocol

  const allowOriginHeader = 'Access-Control-Allow-Origin';

  if (cookie) {
    credentials = 'include';
    headers.push({
      name: 'Access-Control-Allow-Credentials',
      value: 'true',
    });
    headers.push({
      name: allowOriginHeader,
      value: `${location.origin}`,
    });
  } else {
    headers.push({
      name: allowOriginHeader,
      value: '*',
    });
  }
  return {credentials, headers};
};

const getDefaultReportingOrigin = () => {
  // cross-origin means that the reporting origin differs from the source/destination origin.
  const crossOrigin = new URLSearchParams(location.search).get('cross-origin');
  return crossOrigin === null ? location.origin : get_host_info().HTTPS_REMOTE_ORIGIN;
};

const createRedirectChain = (redirects) => {
  let redirectTo;

  for (let i = redirects.length - 1; i >= 0; i--) {
    const {source, trigger, cookie, reportingOrigin} = redirects[i];
    const headers = [];

    if (source) {
      headers.push({
        name: 'Attribution-Reporting-Register-Source',
        value: JSON.stringify(source),
      });
    }

    if (trigger) {
      headers.push({
        name: 'Attribution-Reporting-Register-Trigger',
        value: JSON.stringify(trigger),
      });
    }

    if (cookie) {
      headers.push({name: 'Set-Cookie', value: cookie});
    }

    let status;
    if (redirectTo) {
      headers.push({name: 'Location', value: redirectTo.toString()});
      status = '302';
    }

    redirectTo = blankURLWithHeaders(
        headers, reportingOrigin || getDefaultReportingOrigin(), status);
  }

  return redirectTo;
};

const registerAttributionSrcByImg = (attributionSrc) => {
  const element = document.createElement('img');
  element.attributionSrc = attributionSrc;
};

const registerAttributionSrc = async ({
  source,
  trigger,
  cookie,
  method = 'img',
  extraQueryParams = {},
  reportingOrigin,
  extraHeaders = [],
  referrerPolicy = '',
}) => {
  const searchParams = new URLSearchParams(location.search);

  if (method === 'variant') {
    method = searchParams.get('method');
  }

  const eligible = searchParams.get('eligible');

  let headers = [];

  if (source) {
    headers.push({
      name: 'Attribution-Reporting-Register-Source',
      value: JSON.stringify(source),
    });
  }

  if (trigger) {
    headers.push({
      name: 'Attribution-Reporting-Register-Trigger',
      value: JSON.stringify(trigger),
    });
  }

  if (cookie) {
    const name = 'Set-Cookie';
    headers.push({name, value: cookie});
  }

  let credentials;
  if (method === 'fetch') {
    const params = getFetchParams(reportingOrigin, cookie);
    credentials = params.credentials;
    headers = headers.concat(params.headers);
  }

  headers = headers.concat(extraHeaders);

  const url = blankURLWithHeaders(headers, reportingOrigin);

  Object.entries(extraQueryParams)
      .forEach(([key, value]) => url.searchParams.set(key, value));

  switch (method) {
    case 'img':
      const img = document.createElement('img');
      img.referrerPolicy = referrerPolicy;
      if (eligible === null) {
        img.attributionSrc = url;
      } else {
        await new Promise(resolve => {
          img.onload = resolve;
          // Since the resource being fetched isn't a valid image, onerror will
          // be fired, but the browser will still process the
          // attribution-related headers, so resolve the promise instead of
          // rejecting.
          img.onerror = resolve;
          img.attributionSrc = '';
          img.src = url;
        });
      }
      return 'event';
    case 'script':
      const script = document.createElement('script');
      script.referrerPolicy = referrerPolicy;
      if (eligible === null) {
        script.attributionSrc = url;
      } else {
        await new Promise(resolve => {
          script.onload = resolve;
          script.attributionSrc = '';
          script.src = url;
          document.body.appendChild(script);
        });
      }
      return 'event';
    case 'a':
      const a = document.createElement('a');
      a.referrerPolicy = referrerPolicy;
      a.target = '_blank';
      a.textContent = 'link';
      if (eligible === null) {
        a.attributionSrc = url;
        a.href = blankURL();
      } else {
        a.attributionSrc = '';
        a.href = url;
      }
      document.body.appendChild(a);
      await test_driver.click(a);
      return 'navigation';
    case 'open':
      await test_driver.bless('open window', () => {
        const feature = referrerPolicy === 'no-referrer' ? 'noreferrer' : '';
        if (eligible === null) {
          open(
              blankURL(), '_blank',
              `attributionsrc=${encodeURIComponent(url)} ${feature}`);
        } else {
          open(url, '_blank', `attributionsrc ${feature}`);
        }
      });
      return 'navigation';
    case 'fetch': {
      let attributionReporting;
      if (eligible !== null) {
        attributionReporting = JSON.parse(eligible);
      }
      await fetch(url, {credentials, attributionReporting, referrerPolicy});
      return 'event';
    }
    case 'xhr':
      await new Promise((resolve, reject) => {
        const req = new XMLHttpRequest();
        req.open('GET', url);
        if (eligible !== null) {
          req.setAttributionReporting(JSON.parse(eligible));
        }
        req.onload = resolve;
        req.onerror = () => reject(req.statusText);
        req.send();
      });
      return 'event';
    default:
      throw `unknown method "${method}"`;
  }
};


/**
 * Generates a random pseudo-unique source event id.
 */
const generateSourceEventId = () => {
  return `${Math.round(Math.random() * 10000000000000)}`;
}

/**
 * Delay method that waits for prescribed number of milliseconds.
 */
const delay = ms => new Promise(resolve => step_timeout(resolve, ms));

/**
 * Method that polls a particular URL for reports. Once reports
 * are received, returns the payload as promise. Returns null if the
 * timeout is reached before a report is available.
 */
const pollAttributionReports = async (url, origin = location.origin, timeout = 60 * 1000 /*ms*/) => {
  let startTime = performance.now();
  while (performance.now() - startTime < timeout) {
    const resp = await fetch(new URL(url, origin));
    const payload = await resp.json();
    if (payload.reports.length > 0) {
      return payload;
    }
    await delay(/*ms=*/ 100);
  }
  return null;
};

// Verbose debug reporting must have been enabled on the source registration for this to work.
const waitForSourceToBeRegistered = async (sourceId, reportingOrigin) => {
  const debugReportPayload = await pollVerboseDebugReports(reportingOrigin);
  assert_equals(debugReportPayload.reports.length, 1);
  const debugReport = JSON.parse(debugReportPayload.reports[0].body);
  assert_equals(debugReport.length, 1);
  assert_equals(debugReport[0].type, 'source-success');
  assert_equals(debugReport[0].body.source_event_id, sourceId);
};

const pollEventLevelReports = (origin) =>
    pollAttributionReports(eventLevelReportsUrl, origin);
const pollEventLevelDebugReports = (origin) =>
    pollAttributionReports(eventLevelDebugReportsUrl, origin);
const pollAggregatableReports = (origin) =>
    pollAttributionReports(aggregatableReportsUrl, origin);
const pollAttributionSuccessDebugAggregatableReports = (origin) =>
    pollAttributionReports(attributionSuccessDebugAggregatableReportsUrl, origin);
const pollVerboseDebugReports = (origin) =>
    pollAttributionReports(verboseDebugReportsUrl, origin);
const pollAggregatableDebugReports = (origin) =>
  pollAttributionReports(aggregatableDebugReportsUrl, origin);

const validateReportHeaders = headers => {
  assert_array_equals(headers['content-type'], ['application/json']);
  assert_array_equals(headers['cache-control'], ['no-cache']);
  assert_own_property(headers, 'user-agent');
  assert_not_own_property(headers, 'cookie');
  assert_not_own_property(headers, 'referer');
};
