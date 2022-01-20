async function registerServiceWorkerAndReturnActiveWorker(t, script, scope) {
  const reg = await navigator.serviceWorker.register(script, {scope: scope});
  t.add_cleanup(() => reg.unregister());
  if (reg.active)
    return reg.active;
  const worker =  reg.installing || reg.waiting;
  await new Promise((resolve) => {
    worker.addEventListener('statechange', (event) => {
      if (event.target.state == 'activated')
        resolve();
    });
  });
  return worker;
}

async function getRequestedUrls(worker) {
  return new Promise(resolve => {
    navigator.serviceWorker.addEventListener(
      'message',
      e => {resolve(e.data);},
      {once: true})
    worker.postMessage(null);
  });
}

promise_test(async (t) => {
  const iframe_path = './resources/service-worker-controlled-iframe.html';
  const iframe_url = new URL(iframe_path, location).href

  // Register a service worker.
  const worker = await registerServiceWorkerAndReturnActiveWorker(
      t,
      './resources/service-worker-for-request-monitor.js',
      iframe_path);

  // Load an iframe which is controlled by the service worker.
  const iframe = await new Promise(resolve => {
    const frame = document.createElement('iframe');
    t.add_cleanup(() => frame.remove());
    frame.src = iframe_url;
    frame.onload = () => { resolve(frame); };
    document.body.appendChild(frame);
  });
  // The iframe request should be intercepted by the service worker.
  assert_array_equals(await getRequestedUrls(worker), [iframe_url]);

  // Add a web bundle element in the service worker controlled iframe.
  const frame_id = 'urn:uuid:429fcc4e-0696-4bad-b099-ee9175f023ae';
  const script_id = 'urn:uuid:020111b3-437a-4c5c-ae07-adb6bbffb720';

  const element = createWebBundleElement(
      '../../resources/wbn/urn-uuid.wbn',
      /*resources=*/[frame_id, script_id]);

  const element_load_promise = new Promise(resolve => {
    element.addEventListener('load', () => {resolve();});
  });
  iframe.contentDocument.body.appendChild(element);
  await element_load_promise;
  // The web bundle request should not be intercepted by the service worker.
  assert_array_equals(await getRequestedUrls(worker), []);

  // Add an urn uuid URL script element in the service worker controlled
  // iframe.
  const result_promise = new Promise(resolve => {
    // window.report_result() method will be called by the injected script.
    iframe.contentWindow.report_result = resolve;
  });
  const script = iframe.contentDocument.createElement('script');
  script.src = script_id;
  iframe.contentDocument.body.appendChild(script);
  assert_equals(await result_promise, 'OK');
  // The urn uuld URL script request should not be intercepted by the
  // service worker.
  assert_array_equals(await getRequestedUrls(worker), []);

  // Add an urn uuid URL iframe element in the service worker controlled
  // iframe.
  const inner_iframe = iframe.contentDocument.createElement('iframe');
  inner_iframe.src = frame_id;
  const load_promise = new Promise(resolve => {
    inner_iframe.addEventListener('load', () => {resolve();});
  });
  iframe.contentDocument.body.appendChild(inner_iframe);
  await load_promise;
  // The urn uuld URL iframe request should not intercepted by the service
  // worker.
  assert_array_equals(await getRequestedUrls(worker), []);

  // Check if the urn uuid URL iframe element is loaded correctly.
  const message_promise = new Promise(resolve => {
      window.addEventListener(
          'message',
          e => {resolve(e.data);},
          {once: true});
    });
  // location.href is evaluated in the urn uuid URL iframe element.
  inner_iframe.contentWindow.postMessage('location.href', '*');
  assert_equals(await message_promise, frame_id);
},
'Both Web Bundle request and Subresource fetch requests inside the Web ' +
'Bundle should skip the service worker.');
