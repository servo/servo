// Service worker with 'COEP: require-corp' response header.
// This service worker issues a network request to import scripts with or
// without CORP response header.

importScripts("/common/get-host-info.sub.js");

function url_for_empty_js(corp) {
  const url = new URL(get_host_info().HTTPS_REMOTE_ORIGIN);
  url.pathname = '/service-workers/service-worker/resources/empty.js';
  if (corp) {
    url.searchParams.set(
        'pipe', `header(Cross-Origin-Resource-Policy, ${corp})`);
  }
  return url.href;
}

const params = new URL(location.href).searchParams;

if (params.get('corp') === 'cross-origin') {
  importScripts(url_for_empty_js('cross-origin'));
} else {
  importScripts(url_for_empty_js());
}
