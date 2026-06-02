
const executor_path = '/common/dispatcher/executor.html?pipe=';
const remote_executor_path = '/common/dispatcher/remote-executor.html?pipe=';
const executor_worker_path = '/common/dispatcher/executor-worker.js?pipe=';
const remote_executor_worker_path = '/common/dispatcher/remote-executor-worker.js?pipe=';
const executor_service_worker_path = '/common/dispatcher/executor-service-worker.js?pipe=';

// COEP
const coep_none =
    '|header(Cross-Origin-Embedder-Policy,none)';
const coep_credentialless =
    '|header(Cross-Origin-Embedder-Policy,credentialless)';

// DIP
const dip_none =
    '|header(Document-Isolation-Policy,none)';
const dip_credentialless =
    '|header(Document-Isolation-Policy,isolate-and-credentialless)';
const dip_require_corp =
    '|header(Document-Isolation-Policy,isolate-and-require-corp)';

// DIP-Report-Only
const dip_report_only_credentialless =
    '|header(Document-Isolation-Policy-Report-Only,isolate-and-credentialless)';

// CORP
const corp_cross_origin =
    '|header(Cross-Origin-Resource-Policy,cross-origin)';

const cookie_same_site_none = ';SameSite=None;Secure';

// Test using the modern async/await primitives are easier to read/write.
// However they run sequentially, contrary to async_test. This is the parallel
// version, to avoid timing out.
let promise_test_parallel = (promise, description) => {
  async_test(test => {
    promise(test)
      .then(() => test.done())
      .catch(test.step_func(error => { throw error; }));
  }, description);
};

// Add a cookie |cookie_key|=|cookie_value| on an |origin|.
// Note: cookies visibility depends on the path of the document. Those are set
// from a document from: /html/cross-origin-embedder-policy/credentialless/. So
// the cookie is visible to every path underneath.
const setCookie = async (origin, cookie_key, cookie_value) => {
  const popup_token = token();
  const popup_url = origin + executor_path + `&uuid=${popup_token}`;
  const popup = window.open(popup_url);

  const reply_token = token();
  send(popup_token, `
    document.cookie = "${cookie_key}=${cookie_value}";
    send("${reply_token}", "done");
  `);
  assert_equals(await receive(reply_token), "done");
  popup.close();
}

let parseCookies = function(headers_json) {
  if (!headers_json["cookie"])
    return {};

  return headers_json["cookie"]
    .split(';')
    .map(v => v.split('='))
    .reduce((acc, v) => {
      acc[v[0].trim()] = v[1].trim();
      return acc;
    }, {});
}

// Open a new window with a given |origin|, loaded with DIP:credentialless. The
// new document will execute any scripts sent toward the token it returns.
const newCredentiallessWindow = (origin) => {
  const main_document_token = token();
  const url = origin + executor_path + dip_credentialless +
    `&uuid=${main_document_token}`;
  const context = window.open(url);
  add_completion_callback(() => w.close());
  return main_document_token;
};

// Create a new iframe, loaded with DIP:credentialless.
// The new document will execute any scripts sent toward the token it returns.
const newCredentiallessIframe = (parent_token, child_origin) => {
  const sub_document_token = token();
  const iframe_url = child_origin + executor_path + dip_credentialless +
    `&uuid=${sub_document_token}`;
  send(parent_token, `
    let iframe = document.createElement("iframe");
    iframe.src = "${iframe_url}";
    document.body.appendChild(iframe);
  `)
  return sub_document_token;
};

// The following functions create remote execution contexts with the matching
// origins and headers. The first return value is the uuid that can be used
// to instantiate a RemoteContext object. The second return value is the URL of
// the context that was created.
async function createIframeContext(t, origin, header) {
  const uuid = token();
  const frame_url = origin + remote_executor_path + header + '&uuid=' + uuid;
  const frame = await with_iframe(frame_url);
  t.add_cleanup(() => frame.remove());
  return [uuid, frame_url];
}

async function createDedicatedWorkerContext(t, origin, header) {
  const iframe_uuid = token();
  const frame_url = origin + remote_executor_path + header + '&uuid=' + iframe_uuid;
  const frame = await with_iframe(frame_url);
  t.add_cleanup(() => frame.remove());

  const uuid = token();
  const worker_url = origin + remote_executor_worker_path + '&uuid=' + uuid;
  const ctx = new RemoteContext(iframe_uuid);
  await ctx.execute_script(
    (url) => {
      const worker = new Worker(url);
    }, [worker_url]);
  return [uuid, worker_url];
}

async function createSharedWorkerContext(t, origin, header) {
  const uuid = token();
  const worker_url = origin + remote_executor_worker_path + header + '&uuid=' + uuid;
  const worker = new SharedWorker(worker_url);
  worker.addEventListener('error', t.unreached_func('Worker.onerror'));
  return [uuid, worker_url];
}

async function createIframeWithSWContext(t, origin, header) {
  // Register a service worker with no headers.
  const uuid = token();
  const frame_url = origin + remote_executor_path + header + '&uuid=' + uuid;
  const service_worker_url = origin + executor_service_worker_path;
  const reg = await service_worker_unregister_and_register(
    t, service_worker_url, frame_url);
  const worker = reg.installing || reg.waiting || reg.active;
  worker.addEventListener('error', t.unreached_func('Worker.onerror'));

  const frame = await with_iframe(frame_url);
  t.add_cleanup(() => {
    reg.unregister();
    frame.remove();
  });
  return [uuid, frame_url];
}

// A common interface for building the 4 type of execution contexts. Outputs the
// token needed to create the RemoteContext.
async function getTokenFromEnvironment(t,  environment, headers) {
  switch(environment) {
    case "document":
      const iframe_context = await createIframeContext(t, window.origin, headers);
      return iframe_context[0];
    case "dedicated_worker":
      const dedicated_worker_context = await createDedicatedWorkerContext(t, window.origin, headers);
      return dedicated_worker_context[0];
    case "shared_worker":
      const shared_worker_context = await createSharedWorkerContext(t, window.origin, headers);
      return shared_worker_context[0];
    case "service_worker":
      const sw_context = await createIframeWithSWContext(t, window.origin, headers);
      return sw_context[0];
  }
}

// A common interface for building the 4 type of execution contexts:
// It outputs: [
//   - The token to communicate with the environment.
//   - A promise resolved when the environment encounters an error.
// ]
const environments = {
  document: headers => {
    const tok = token();
    const url = window.origin + executor_path + headers + `&uuid=${tok}`;
    const context = window.open(url);
    add_completion_callback(() => context.close());
    return [tok, new Promise(resolve => {})];
  },

  dedicated_worker: headers => {
    const tok = token();
    const url = window.origin + executor_worker_path + headers + `&uuid=${tok}`;
    const context = new Worker(url);
    return [tok, new Promise(resolve => context.onerror = resolve)];
  },

  shared_worker: headers => {
    const tok = token();
    const url = window.origin + executor_worker_path + headers + `&uuid=${tok}`;
    const context = new SharedWorker(url);
    return [tok, new Promise(resolve => context.onerror = resolve)];
  },

  service_worker: headers => {
    const tok = token();
    const url = window.origin + executor_worker_path + headers + `&uuid=${tok}`;
    const scope = url; // Generate a one-time scope for service worker.
    const error = new Promise(resolve => {
      navigator.serviceWorker.register(url, {scope: scope})
        .then(registration => {
          add_completion_callback(() => registration.unregister());
        }, /* catch */ resolve);
    });
    return [tok, error];
  },
};
