// This test tries to load 'script.js' subresource from a static-element.wbn,
// using a relative URL instead of an absolute one with a <link> and <script>
// elements directly in the document (they are used only for this test).
promise_test(async () => {
  assert_equals(resources_script_result, 'loaded from webbundle');
},
'A subresource script.js should be loaded from WebBundle using the relative ' +
'URL.');

// Simple load of a root.js subresource from subresource.wbn using a relative
// URL.
promise_test(async () => {
  const resource_url = '/web-bundle/resources/wbn/root.js';

  const element = createWebBundleElement(
      '../resources/wbn/subresource.wbn',
      [resource_url]);
  document.body.appendChild(element);

  const response = await fetch(resource_url);
  assert_true(response.ok);
  const root = await response.text();
  assert_equals(root, 'export * from \'./submodule.js\';\n');
}, 'Subresources with relative URLs should be loaded from the WebBundle.');

// Simple load of a root.js subresource from subresource.wbn using an
// incorrect relative URL leading to a failed fetch.
promise_test(async () => {
  const resource_url = 'web-bundle/resources/wbn/root.js';

  const element = createWebBundleElement(
    '../resources/wbn/subresource.wbn',
    [resource_url]);
  document.body.appendChild(element);

  const response = await fetch(resource_url);
  assert_false(response.ok);
}, 'Wrong relative URL should result in a failed fetch.');

// Simple load of subresources under a scope from dynamic1.wbn using a relative
// URL.
promise_test(async () => {
  const resource_scope = '/web-bundle/resources/wbn/dynamic/resource';

  const element = createWebBundleElement(
      '../resources/wbn/dynamic1.wbn',
      [], {scopes: [resource_scope]});
  document.body.appendChild(element);

  const module = await import('/web-bundle/resources/wbn/dynamic/resource1.js');
  assert_equals(module.result, 'resource1 from dynamic1.wbn');
  const module2 = await import('/web-bundle/resources/wbn/dynamic/resource2.js');
  assert_equals(module2.result, 'resource2 from dynamic1.wbn');
  const module3 = await import('/web-bundle/resources/wbn/dynamic/resource3.js');
  assert_equals(module3.result, 'resource3 from dynamic1.wbn');
  const module4 = await import('/web-bundle/resources/wbn/dynamic/resource4.js');
  assert_equals(module4.result, 'resource4 from dynamic1.wbn');
}, 'Subresources inside the scope specified with relative URL should be loaded from the WebBundle.');

// Simple load of subresources under a scope from dynamic1.wbn using a relative
// URL. As the scope URL is wrong, the fetch should fail.
promise_test(async () => {
  const resource_scope = '/web-bundle/resources/wbn/dynami/';

  const element = createWebBundleElement(
      '../resources/wbn/dynamic1.wbn',
      [], {scopes: [resource_scope]});
  document.body.appendChild(element);

  const result_promise = new Promise((resolve) => {
    // This function will be called from script.js
    window.report_result = resolve;
  });

  const script = document.createElement('script');
  script.src = '/web-bundle/resources/wbn/dynamic/classic_script.js';
  document.body.appendChild(script);
  assert_equals(await result_promise, 'classic script from network');
}, 'No subresources should be loaded from the bundle when the relative url of the scope is wrong');
