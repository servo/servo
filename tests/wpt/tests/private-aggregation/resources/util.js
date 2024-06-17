// In a shared storage worklet initialized via
// sharedStorage.worklet.addModule(), execute Private Aggregation functions
// given `paa_data`, and expect that success/failure result is `expected_error`.
async function VerifyContributeToHistogram(paa_data, expected_error) {
  const ancestor_key = token();
  let url0 = generateURL("/shared-storage/resources/frame0.html",
                         [ancestor_key]);
  let url1 = generateURL("/shared-storage/resources/frame1.html",
                         [ancestor_key]);

  await addModuleOnce("/private-aggregation/resources/shared-storage-helper-module.js");

  let select_url_result = await sharedStorage.selectURL(
    "contribute-to-histogram", [{url: url0}, {url: url1}],
    {data: paa_data, keepAlive: true});

  attachFencedFrame(select_url_result, 'opaque-ads');
  const result = await nextValueFromServer(ancestor_key);

  if (expected_error) {
    assert_equals(result, "frame0_loaded");
  } else {
    assert_equals(result, "frame1_loaded");
  }
}

// In a shared storage worklet created from sharedStorage.createWorklet() with
// the given `shared_storage_origin`, execute Private Aggregation functions
// given `paa_data`, and expect that success/failure result is `expected_error`.
// Same-origin script will be used if `shared_storage_origin` is empty.
async function CreateWorkletAndVerifyContributeToHistogram(shared_storage_origin, paa_data, expected_error) {
  const ancestor_key = token();
  let url0 = generateURL("/shared-storage/resources/frame0.html",
                         [ancestor_key]);
  let url1 = generateURL("/shared-storage/resources/frame1.html",
                         [ancestor_key]);

  let worklet = await sharedStorage.createWorklet(shared_storage_origin +
          "/private-aggregation/resources/shared-storage-helper-module.js");

  let select_url_result = await worklet.selectURL(
    "contribute-to-histogram", [{url: url0}, {url: url1}],
    {data: paa_data, keepAlive: true});

  attachFencedFrame(select_url_result, 'opaque-ads');
  const result = await nextValueFromServer(ancestor_key);

  if (expected_error) {
    assert_equals(result, "frame0_loaded");
  } else {
    assert_equals(result, "frame1_loaded");
  }
}

