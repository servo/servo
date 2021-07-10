const directory = '/html/cross-origin-embedder-policy/credentialless';
const executor_path = directory + '/resources/executor.html?pipe=';

// COEP
const coep_none =
    '|header(Cross-Origin-Embedder-Policy,none)';
const coep_credentialless =
    '|header(Cross-Origin-Embedder-Policy,credentialless)';
const coep_require_corp =
    '|header(Cross-Origin-Embedder-Policy,require-corp)';

// COOP
const coop_same_origin =
    '|header(Cross-Origin-Opener-Policy,same-origin)';

// CORP
const corp_cross_origin =
    '|header(Cross-Origin-Resource-Policy,cross-origin)';

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

let parseCookies = function(headers_json) {
  if (!headers_json["cookie"])
    return {};

  return headers_json["cookie"]
    .split(';')
    .map(v => v.split('='))
    .reduce((acc, v) => {
      acc[v[0]] = v[1];
      return acc;
    }, {});
}

// Open a new window with a given |origin|, loaded with COEP:credentialless. The
// new document will execute any scripts sent toward the token it returns.
const newCredentiallessWindow = (origin) => {
  const main_document_token = token();
  const url = origin + executor_path + coep_credentialless +
    `&uuid=${main_document_token}`;
  const w = window.open(url);
  add_completion_callback(() => w.close());
  return main_document_token;
};

// Create a new iframe, loaded with COEP:credentialless.
// The new document will execute any scripts sent toward the token it returns.
const newCredentiallessIframe = (parent_token, child_origin) => {
  const sub_document_token = token();
  const iframe_url = child_origin + executor_path + coep_credentialless +
    `&uuid=${sub_document_token}`;
  send(parent_token, `
    let iframe = document.createElement("iframe");
    iframe.src = "${iframe_url}";
    document.body.appendChild(iframe);
  `)
  return sub_document_token;
};

