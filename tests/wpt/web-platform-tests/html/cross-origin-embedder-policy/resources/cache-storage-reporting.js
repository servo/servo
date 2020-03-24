function remote(path) {
  const REMOTE_ORIGIN = get_host_info().HTTPS_REMOTE_ORIGIN;
  return new URL(path, REMOTE_ORIGIN);
}

function local(path) {
  return new URL(path, location.origin);
}

let encode = function(url) {
  return encodeURI(url).replace(/\;/g,"%3B");
}

const resource_path = (new URL("./resources", location)).pathname;
const report_token= token();
const report_endpoint_url = local(resource_path + `/report.py?key=${report_token}`)
const endpoint =
{
   "group":"endpoint",
   "max_age":3600,
   "endpoints":[{ "url":report_endpoint_url.toString() }]
};
let endpoint_string =
  JSON.stringify(endpoint)
   .replace(/,/g, "\\,")
   .replace(/\(/g, "\\\(")
   .replace(/\)/g, "\\\)=");
const header_report_to = `|header(report-to,${endpoint_string})`;
const header_coep =
  '|header(Cross-Origin-Embedder-Policy,require-corp;report-to="endpoint")';
const header_coep_report_only =
  '|header(Cross-Origin-Embedder-Policy-Report-Only,require-corp;report-to="endpoint")';
const SW_SCOPE = local(resource_path + "/");
const header_service_worker_allowed =
  `|header(service-worker-allowed,${SW_SCOPE})`;

const iframe_path = resource_path + "/iframe.html?pipe=";
const worker_path = resource_path + "/universal-worker.js?pipe=";
const image_url = remote("/images/blue.png");

// This script attempt to load a COEP:require-corp CORP:undefined response from
// the CacheStorage.
//
// Executed from different context:
// - A Document
// - A ServiceWorker
// - A DedicatedWorker
// - A SharedWorker
//
// The context has either COEP or COEP-Report-Only defined.
const eval_script = `
  (async function() {
    try {
      const cache = await caches.open('v1');
      const request = new Request('${image_url}', { mode: 'no-cors' });
      const response = await cache.match(request);
    } catch(e) {
    }
  })()
`;

promise_setup(async (t) => {
  const cache = await caches.open('v1');
  const fetch_request = new Request(image_url, {mode: 'no-cors'});
  const fetch_response = await fetch(fetch_request);
  await cache.put(fetch_request, fetch_response);
}, "Setup: store a CORS:cross-origin COEP:none response into CacheStorage")

async function makeIframe(test, iframe_url) {
  const iframe = document.createElement("iframe");
  test.add_cleanup(() => iframe.remove());
  iframe.src = iframe_url;
  const iframe_loaded = new Promise(resolve => iframe.onload = resolve);
  document.body.appendChild(iframe);
  await iframe_loaded;
  return iframe;
}

function wait(ms) {
  return new Promise(resolve => step_timeout(resolve, ms));
}

async function fetchReport() {
  const fetch_report_path = resource_path + `/report.py?key=${report_token}`;
  while(true) {
    const response = await fetch(encode(fetch_report_path));
    const reports = await response.json();
    if (reports.length == 0) {
      wait(200);
      continue;
    }
    if (reports.length != 1)
      throw "Too many reports received";

    return reports[0];
  }
  throw "Report not send";
}

// Remove parts of the URL that are different at runtime.
function normalize(url) {
  url = new URL(url);
  return url.origin + url.pathname;
}
