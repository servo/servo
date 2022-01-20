promise_test(async () => {
  const resources = [
    "/web-bundle/resources/wbn/resource.js",
    "/web-bundle/resources/wbn/sub/resource.js",
  ];
  for (const resource of resources) {
    const response = await fetch(resource);
    assert_true(response.ok, resource + " should be loaded");
  }
}, "Subresources should be loaded.");

promise_test(async () => {
  const resources = [
    "/web-bundle/resources/wbn-resource.js",
    "/web-bundle/resources/wbn1/resource.js",
    "/web-bundle/resources/other/resource.js",
    "/web-bundle/resources/resource.js",
  ];
  for (const resource of resources) {
    const response = await fetch(resource);
    assert_false(response.ok, resource + " should not be loaded");
  }
}, "Subresources should not be loaded due to path restriction.");