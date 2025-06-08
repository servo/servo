// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../resources/soft-navigation-test-helper.js

// This test shows a relatively simple case where a div with an image inside
// is inserted before another element, and the image is painted, and yet
// we don't detect a soft navigation.
// https://g-issues.chromium.org/issues/419822831#comment5

function clickHandler() {
  const div = document.createElement("div");
  const img = new Image();
  img.src = "/images/lcp-256x256.png"
  // Uncomment the following line => test passes (image should work too though).
  // div.textContent = "Hello, World.";
  div.appendChild(img);
  document.body.insertBefore(div, document.getElementById("insert-before"));
  history.pushState({}, '', '/test');
}

const div = document.createElement('div');
div.id = 'insert-before';
document.body.appendChild(div);

const button = document.createElement('div');
button.textContent = 'Click here!';
button.onclick = clickHandler;
document.body.appendChild(button);

promise_test(async (t) => {
  if (test_driver) {
    test_driver.click(button);
  }
  const helper = new SoftNavigationTestHelper(t);
  const entries = await helper.getBufferedPerformanceEntriesWithTimeout(
      /*type=*/ 'soft-navigation',
      /*includeSoftNavigationObservations=*/ false,
      /*minNumEntries=*/ 1,
      /*timeout=*/ 3000,
  );
  assert_equals(entries.length, 1, 'Expected exactly one soft navigation.');
  assert_equals(
      entries[0].name.replace(/.*\//, ''),
      'test',
      'URL ends with \'test\'.',
  );
}, 'DOM: Insert image div satisfies Soft Navigation paint criterion.');
