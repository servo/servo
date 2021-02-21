const directory = '/html/cross-origin-embedder-policy/credentialless';

const executor_path = directory + '/resources/executor.html?pipe=';
const coep_none = '|header(Cross-Origin-Embedder-Policy,none)';
const coep_credentialless =
    '|header(Cross-Origin-Embedder-Policy,credentialless)';

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
