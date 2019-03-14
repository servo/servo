const reportID = "{{$id:uuid()}}";

/*
 * NEL tests have to run serially, since the user agent maintains a global cache
 * of Reporting and NEL policies, and we don't want the policies for multiple
 * tests to interfere with each other.  These functions (along with a Python
 * handler in lock.py) implement a simple spin lock.
 */

function obtainNELLock() {
  return fetch("/network-error-logging/support/lock.py?op=lock&reportID=" + reportID);
}

function releaseNELLock() {
  return fetch("/network-error-logging/support/lock.py?op=unlock&reportID=" + reportID);
}

function nel_test(callback, name, properties) {
  promise_test(async t => {
    await obtainNELLock();
    await clearReportingAndNELConfigurations();
    await callback(t);
    await releaseNELLock();
  }, name, properties);
}

/*
 * Helper functions for constructing domain names that contain NEL policies.
 */
function _monitoredDomain(subdomain) {
  if (subdomain == "www") {
    return "{{hosts[alt][www]}}"
  } else if (subdomain == "www1") {
    return "{{hosts[alt][www1]}}"
  } else if (subdomain == "www2") {
    return "{{hosts[alt][www2]}}"
  } else if (subdomain == "nonexistent") {
    return "{{hosts[alt][nonexistent]}}"
  } else {
    return "{{hosts[alt][]}}"
  }
}

function _getNELResourceURL(subdomain, suffix) {
  return "https://" + _monitoredDomain(subdomain) +
    ":{{ports[https][0]}}/network-error-logging/support/" + suffix;
}

/*
 * Fetches a resource whose headers define a basic NEL policy (i.e., with no
 * include_subdomains flag).  We ensure that we request the resource from a
 * different origin than is used for the main test case HTML file or for report
 * uploads.  This minimizes the number of reports that are generated for this
 * policy.
 */

function getURLForResourceWithBasicPolicy(subdomain) {
  return _getNELResourceURL(subdomain, "pass.png?id="+reportID+"&success_fraction=1.0");
}

function fetchResourceWithBasicPolicy(subdomain) {
  const url = getURLForResourceWithBasicPolicy(subdomain);
  return fetch(url, {mode: "no-cors"});
}

function fetchResourceWithZeroSuccessFractionPolicy(subdomain) {
  const url = _getNELResourceURL(subdomain, "pass.png?id="+reportID+"&success_fraction=0.0");
  return fetch(url, {mode: "no-cors"});
}

/*
 * Fetches a resource whose headers define an include_subdomains NEL policy.
 */

function getURLForResourceWithIncludeSubdomainsPolicy(subdomain) {
  return _getNELResourceURL(subdomain, "subdomains-pass.png?id="+reportID);
}

function fetchResourceWithIncludeSubdomainsPolicy(subdomain) {
  const url = getURLForResourceWithIncludeSubdomainsPolicy(subdomain);
  return fetch(url, {mode: "no-cors"});
}

/*
 * Fetches a resource whose headers do NOT define a NEL policy.  This may or may
 * not generate a NEL report, depending on whether you've already successfully
 * requested a resource from the same origin that included a NEL policy.
 */

function getURLForResourceWithNoPolicy(subdomain) {
  return _getNELResourceURL(subdomain, "no-policy-pass.png");
}

function fetchResourceWithNoPolicy(subdomain) {
  const url = getURLForResourceWithNoPolicy(subdomain);
  return fetch(url, {mode: "no-cors"});
}

/*
 * Fetches a resource that doesn't exist.  This may or may not generate a NEL
 * report, depending on whether you've already successfully requested a resource
 * from the same origin that included a NEL policy.
 */

function getURLForMissingResource(subdomain) {
  return _getNELResourceURL(subdomain, "nonexistent.png");
}

function fetchMissingResource(subdomain) {
  const url = getURLForMissingResource(subdomain);
  return fetch(url, {mode: "no-cors"});
}

/*
 * Fetches a resource that can be cached without validation.
 */

function getURLForCachedResource(subdomain) {
  return _getNELResourceURL(subdomain, "cached-for-one-minute.png");
}

function fetchCachedResource(subdomain) {
  const url = getURLForCachedResource(subdomain);
  return fetch(url, {mode: "no-cors"});
}

/*
 * Fetches a resource that can be cached but requires validation.
 */

function getURLForValidatedCachedResource(subdomain) {
  return _getNELResourceURL(subdomain, "cached-with-validation.py");
}

function fetchValidatedCachedResource(subdomain) {
  const url = getURLForValidatedCachedResource(subdomain);
  return fetch(url, {mode: "no-cors"});
}

/*
 * Fetches a resource that redirects once before returning a successful
 * response.
 */

function getURLForRedirectedResource(subdomain) {
  return _getNELResourceURL(subdomain, "redirect.py?id="+reportID);
}

function fetchRedirectedResource(subdomain) {
  const url = getURLForRedirectedResource(subdomain);
  return fetch(url, {mode: "no-cors"});
}

/*
 * Fetches resources that clear out any existing Reporting or NEL configurations
 * for all origins that any test case might use.
 */

function getURLForClearingConfiguration(subdomain) {
  return _getNELResourceURL(subdomain, "clear-policy-pass.png?id="+reportID);
}

async function clearReportingAndNELConfigurations(subdomain) {
  await Promise.all([
    fetch(getURLForClearingConfiguration(""), {mode: "no-cors"}),
    fetch(getURLForClearingConfiguration("www"), {mode: "no-cors"}),
    fetch(getURLForClearingConfiguration("www1"), {mode: "no-cors"}),
    fetch(getURLForClearingConfiguration("www2"), {mode: "no-cors"}),
  ]);
  return;
}

/*
 * Returns whether all of the fields in obj1 also exist in obj2 with the same
 * values.  (Put another way, returns whether obj1 and obj2 are equal, ignoring
 * any extra fields in obj2.)
 */

function _isSubsetOf(obj1, obj2) {
  for (const prop in obj1) {
    if (typeof obj1[prop] === 'object') {
      if (typeof obj2[prop] !== 'object') {
        return false;
      }
      if (!_isSubsetOf(obj1[prop], obj2[prop])) {
        return false;
      }
    } else if (obj1[prop] != obj2[prop]) {
      return false;
    }
  }
  return true;
}

/*
 * Verifies that a report was uploaded that contains all of the fields in
 * expected.
 */

async function reportExists(expected) {
  var timeout =
    document.querySelector("meta[name=timeout][content=long]") ? 50 : 1;
  var reportLocation =
    "/network-error-logging/support/report.py?op=retrieve_report&timeout=" +
    timeout + "&reportID=" + reportID;
  const response = await fetch(reportLocation);
  const json = await response.json();
  for (const report of json) {
    if (_isSubsetOf(expected, report)) {
      return true;
    }
  }
  return false;
}

/*
 * Verifies that reports were uploaded that contains all of the fields in
 * expected.
 */

async function reportsExist(expected_reports) {
  const timeout = 10;
  let reportLocation =
    "/network-error-logging/support/report.py?op=retrieve_report&timeout=" +
    timeout + "&reportID=" + reportID;
  // There must be the report of pass.png, so adding 1.
  const min_count = expected_reports.length + 1;
  reportLocation += "&min_count=" + min_count;
  const response = await fetch(reportLocation);
  const json = await response.json();
  for (const expected of expected_reports) {
    const found = json.some((report) => {
      return _isSubsetOf(expected, report);
    });
    if (!found)
      return false;
  }
  return true;
}
