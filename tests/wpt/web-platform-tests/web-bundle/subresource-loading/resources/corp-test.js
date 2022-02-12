
promise_test(async () => {
  const prefix = 'https://www1.web-platform.test:8444/web-bundle/resources/wbn/cors/';
  await addScriptAndWaitForExecution(prefix + 'no-corp.js');
  await addScriptAndWaitForError(prefix + 'corp-same-origin.js');
  await addScriptAndWaitForExecution(prefix + 'corp-cross-origin.js');
}, "Subresource loading from WebBundles should respect Cross-Origin-Resource-Policy header.");

promise_test(async () => {
  const no_corp_url = 'urn:uuid:5eafff38-e0a0-4661-bde0-434255aa9d93';
  const corp_same_origin_url = 'urn:uuid:7e13b47a-8b91-4a0e-997c-993a5e2f3a34';
  const corp_cross_origin_url = 'urn:uuid:86d5b696-8867-4454-8b07-51239a0817f7';
  await iframeLocationTest(no_corp_url);
  await iframeLocationTest(corp_same_origin_url);
  await iframeLocationTest(corp_cross_origin_url);
}, "Urn:uuid iframes should not be blocked regardless of the Cross-Origin-Resource-Policy header, if Cross-Origin-Embedder-Policy is not set.");

async function iframeLocationTest(url) {
  const iframe = document.createElement('iframe');
  iframe.src = url;
  await addElementAndWaitForLoad(iframe);
  assert_equals(await evalInIframe(iframe, 'location.href'), url);
}
