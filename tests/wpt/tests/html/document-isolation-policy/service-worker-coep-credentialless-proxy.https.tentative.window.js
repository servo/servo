// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js

const same_origin = get_host_info().HTTPS_ORIGIN;
const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;

promise_test(async test => {
  const this_token_1 = token();
  const this_token_2 = token();

  // Register a COEP:credentialless ServiceWorker.
  const sw_token = token();
  const sw_url =
    executor_service_worker_path + coep_credentialless + `&uuid=${sw_token}`;
  // Executors should be controlled by the service worker.
  const scope = executor_path;
  const sw_registration =
    await service_worker_unregister_and_register(test, sw_url, scope);
  test.add_cleanup(() => sw_registration.unregister());
  await wait_for_state(test, sw_registration.installing, 'activated');

  // Configure the ServiceWorker to proxy the fetch requests. Wait for the
  // worker to be installed and activated.
  send(sw_token, `
    fetchHandler = event => {
      if (!event.request.url.includes("/proxied"))
        return;

      send("${this_token_1}", "ServiceWorker: Proxying");

      // Response with a cross-origin no-cors resource.
      const url = "${cross_origin}" + "/common/blank.html";

      event.respondWith(new Promise(async resolve => {
        try {
          let response = await fetch(url, {
            mode: "no-cors",
            credentials: "include"
          });
          send("${this_token_1}", "ServiceWorker: Fetch success");
          resolve(response);
        } catch (error) {
          send("${this_token_1}", "ServiceWorker: Fetch failure");
          resolve(new Response("", {status: 400}));
        }
      }));
    }

    await clients.claim();

    send("${this_token_1}", serviceWorker.state);
  `)
  assert_equals(await receive(this_token_1), "activated");

  // Create a DIP:credentialless document.
  const document_token = environments["document"](dip_credentialless)[0];

  // The document fetches a same-origin no-cors resource. The requests needs to
  // be same-origin to be handled by the ServiceWorker.
  send(document_token, `
    try {
      const response = await fetch("/proxied", { mode: "no-cors", });

      send("${this_token_2}", "Document: Fetch success");
    } catch (error) {
      send("${this_token_2}", "Document: Fetch error");
    }
  `);

  // The COEP:credentialless ServiceWorker is able to handle the cross-origin
  // no-cors request, requested with credentials.
  assert_equals(await receive(this_token_1), "ServiceWorker: Proxying");
  assert_equals(await receive(this_token_1), "ServiceWorker: Fetch success");

  // The COEP:credentialless Document is allowed by CORP to get it.
  assert_equals(await receive(this_token_2), "Document: Fetch success");

  // test.add_cleanup doesn't allow waiting for a promise. Unregistering a
  // ServiceWorker is an asynchronous operation. It might not be completed on
  // time for the next test. Do it here for extra flakiness safety.
  await sw_registration.unregister()
}, "COEP:credentialless ServiceWorker");
