// This is a helper file used for the attribution-reporting-*.https.html tests.
// To use this, make sure you import these scripts:
// <script src="/resources/testharness.js"></script>
// <script src="/resources/testharnessreport.js"></script>
// <script src="/common/utils.js"></script>
// <script src="/common/dispatcher/dispatcher.js"></script>
// <script src="resources/utils.js"></script>
// <script src="/common/get-host-info.sub.js"></script>

async function runDefaultEnabledFeaturesTest(t, should_load, fenced_origin,
    generator_api="fledge", allow="") {
  const fencedframe = await attachFencedFrameContext({
      generator_api: generator_api,
      attributes: [["allow", allow]],
      origin: fenced_origin});

  if (!should_load) {
    const fencedframe_blocked = new Promise(r => t.step_timeout(r, 1000));
    const fencedframe_loaded = fencedframe.execute(() => {});
    assert_equals(await Promise.any([
      fencedframe_blocked.then(() => "blocked"),
      fencedframe_loaded.then(() => "loaded"),
    ]), "blocked", "The fenced frame should not be loaded.");
    return;
  }

  await fencedframe.execute((generator_api) => {
    assert_true(
        document.featurePolicy.allowsFeature('attribution-reporting'),
        "Attribution reporting should be allowed if the fenced " +
        "frame loaded using FLEDGE or shared storage.");

    if (generator_api == "fledge") {
      assert_true(
            document.featurePolicy.allowsFeature('shared-storage'),
            "Shared Storage should be allowed if the fenced " +
            "frame loaded using FLEDGE.");
      assert_true(
            document.featurePolicy.allowsFeature('private-aggregation'),
            "Private Aggregation should be allowed if the fenced " +
            "frame loaded using FLEDGE.");
    } else {
      assert_true(
            document.featurePolicy.allowsFeature('shared-storage'),
            "Shared Storage should be allowed if the fenced " +
            "frame loaded using Shared Storage.");
      assert_false(
            document.featurePolicy.allowsFeature('private-aggregation'),
            "Private Aggregation should be disabled if the fenced " +
            "frame loaded using Shared Storage.");
    }
  }, [generator_api]);
}
