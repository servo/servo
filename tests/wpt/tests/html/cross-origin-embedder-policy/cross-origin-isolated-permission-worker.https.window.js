// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./credentialless/resources/common.js
// META: script=./resources/common.js

const cors_coep_headers = coep_require_corp + corp_cross_origin;
const same_origin = get_host_info().HTTPS_ORIGIN;
const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
const dedicatedWorkerPostMessage = `
  self.addEventListener('message', (e) => {
    e.data.port.postMessage(self.crossOriginIsolated);
  });
`;

const postMessageIsWorkerCrossOriginIsolated = async (
  test,
  frame,
  worker_url
) => {
  const worker = new frame.contentWindow.Worker(worker_url);
  const mc = new MessageChannel();
  worker.postMessage({port: mc.port2}, [mc.port2]);
  worker.onerror = test.unreached_func('cannot create dedicated worker');
  return (await new Promise(r => mc.port1.onmessage = r)).data;
}

const isDataDedicatedWorkerCrossOriginIsolated = async (
  test,
  parent_headers
) => {
  const [future_child, future_error] =
    await createIsolatedFrame('', parent_headers);
  future_error.then(test.unreached_func('cannot create isolated iframe'));

  const child = await future_child;
  add_completion_callback(() => child.remove());

  const worker_url =
    `data:application/javascript;base64,${btoa(dedicatedWorkerPostMessage)}`;
  return postMessageIsWorkerCrossOriginIsolated(test, child, worker_url);
}

const isBlobURLDedicatedWorkerCrossOriginIsolated = async(
  test,
  parent_headers
) => {
  const [future_child, future_error] =
    await createIsolatedFrame("", parent_headers);
  future_error.then(test.unreached_func('cannot create isolated iframe'));

  const child = await future_child;
  add_completion_callback(() => child.remove());

  const blob =
    new Blob([dedicatedWorkerPostMessage], {type: 'text/plaintext'});
  const workerURL = URL.createObjectURL(blob);
  return postMessageIsWorkerCrossOriginIsolated(test, child, workerURL)
}

const isHTTPSDedicatedWorkerCrossOriginIsolated = async(
  test,
  parent_headers
) => {
  const [future_child, future_error] =
    await createIsolatedFrame("", parent_headers);
  future_error.then(test.unreached_func('cannot create isolated iframe'));

  const child = await future_child;
  add_completion_callback(() => child.remove());

  const worker_token = token();
  const workerURL =
    `${executor_worker_path}${cors_coep_headers}&uuid=${worker_token}`;
  const worker = new child.contentWindow.Worker(workerURL);
  return IsCrossOriginIsolated(worker_token);
}

const sharedWorkerIsCrossOriginIsolated = async(
  test,
  withCoopCoep
) => {
  const [worker, future_error] =
    environments.shared_worker(withCoopCoep ? cors_coep_headers : "");
  future_error.then(test.unreached_func('cannot create shared worker.'));
  return IsCrossOriginIsolated(worker);
}

const serviceWorkerIsCrossOriginIsolated = async(
  test,
  withCoopCoep
) => {
  const [worker, future_error] =
    environments.service_worker(withCoopCoep ? cors_coep_headers : "");
  future_error.then(test.unreached_func('cannot create service worker.'));
  return IsCrossOriginIsolated(worker);
}

const dedicatedWorkerIsCrossOriginIsolated = async (
  test,
  scheme,
  parent_permission_coi
) => {
  let parent_headers = cors_coep_headers;
  if (parent_permission_coi !== undefined) {
    // Escape right parenthesis in WPT cors_coep_headers:
    parent_permission_coi = parent_permission_coi.replace(')', '\\)');
    parent_headers += `|header(permissions-policy,` +
                      `cross-origin-isolated=${parent_permission_coi})`;
  }
  switch (scheme) {
    case 'https':
      return isHTTPSDedicatedWorkerCrossOriginIsolated(test, parent_headers);
    case 'data':
      return isDataDedicatedWorkerCrossOriginIsolated(test, parent_headers);
    case 'blob':
      return isBlobURLDedicatedWorkerCrossOriginIsolated(test, parent_headers);
    default:
      assert_unreached("wrong scheme for dedicated worker test.");
  }
}

const generate_shared_worker_test = async (withCoopCoep, expected) => {
  promise_test_parallel(async (test) => {
  const isCrossOriginIsolated =
    await sharedWorkerIsCrossOriginIsolated(test, withCoopCoep);
  assert_equals(isCrossOriginIsolated, expected)
  }, `shared_worker (withCoopCoep: ${withCoopCoep}) ` +
     `cross origin isolated permission test`);
}

const generate_dedicated_worker_test = async (
  scheme,
  parent_permission_coi,
  expected
) => {
  promise_test_parallel(async (test) => {
  const isCrossOriginIsolated =
    await dedicatedWorkerIsCrossOriginIsolated(test, scheme, parent_permission_coi);
  assert_equals(isCrossOriginIsolated, expected)
  }, `dedicated_worker (scheme: ${scheme}) cross origin ` +
     `isolated (${parent_permission_coi}) permission test`);
}

const generate_service_worker_test = async (withCoopCoep, expected) => {
  promise_test_parallel(async (test) => {
  const isCrossOriginIsolated =
    await serviceWorkerIsCrossOriginIsolated(test, withCoopCoep);
  assert_equals(isCrossOriginIsolated, expected)
  }, `service_worker (withCoopCoep: ${withCoopCoep}) ` +
     `cross origin isolated permission test`);
}

generate_shared_worker_test(false, false);
generate_shared_worker_test(true, true);

generate_dedicated_worker_test('https', undefined, true);
generate_dedicated_worker_test('https', '*', true);
generate_dedicated_worker_test('https', 'self', true);
generate_dedicated_worker_test('https', '()', false);
generate_dedicated_worker_test('data', undefined, false);
generate_dedicated_worker_test('data', '*', false);
generate_dedicated_worker_test('data', 'self', false);
generate_dedicated_worker_test('blob', undefined, true);
generate_dedicated_worker_test('blob', '*', true);
generate_dedicated_worker_test('blob', 'self', true);
generate_dedicated_worker_test('blob', '()', false);

generate_service_worker_test(false, false);
generate_service_worker_test(true, true);