function remote(path) {
  const REMOTE_ORIGIN = get_host_info().HTTPS_REMOTE_ORIGIN;
  return new URL(path, REMOTE_ORIGIN).href;
}

function local(path) {
  return new URL(path, location.origin).href;
}

function encode(url) {
  return encodeURI(url).replace(/\;/g, '%3B');
}

const resource_path = (new URL('./resources', location)).pathname;
const header_coep = '|header(Cross-Origin-Embedder-Policy,require-corp)';
const header_coep_report_only =
    '|header(Cross-Origin-Embedder-Policy-Report-Only,require-corp)';

const iframe_path = resource_path + '/iframe.html?pipe=';
const worker_path = resource_path + '/reporting-worker.js?pipe=';
const image_url = remote('/images/blue.png');

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
  const request = new Request(image_url, {mode: 'no-cors'});
  const response = await fetch(request);
  await cache.put(request, response);
}, 'Setup: store a CORS:cross-origin COEP:none response into CacheStorage')

async function makeIframe(test, iframe_url) {
  const iframe = document.createElement('iframe');
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
