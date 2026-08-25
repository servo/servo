// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../resources/soft-navigation-test-helper.js

// This test is very similar to distant-leaf-text.window.js, but the leaf
// node is an image instead of text.
//
// This test is intended to verify that when a distant leaf of a
// deeply nested div element is attached to the DOM, its painting can
// trigger a soft navigation.
//
// To show this, we create a button that, when clicked, creates a deeply
// nested div element and attaches it to the DOM - only the leaf, an image,
// 10 levels below the attachment point actually gets painted.
//
// An earlier version of this test was based on
// https://g-issues.chromium.org/issues/419822831#comment5

function clickHandler() {
  let div = document.createElement("div");
  const img = new Image();  // The leaf node that gets painted.
  img.src = "/images/lcp-256x256.png"
  div.appendChild(img);
  for (let i = 0; i < 10; i++) {
    const tmp = document.createElement('div');
    tmp.appendChild(div);
    div = tmp;
  }
  document.body.appendChild(div);
  history.pushState({}, '', '/leaf-image');
}

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
      /*minNumEntries=*/ 1,
      /*timeout=*/ 3000
  );
  assert_equals(entries.length, 1, 'Expected exactly one soft navigation.');
  assert_equals(
      entries[0].name.replace(/.*\//, ''),
      'leaf-image',
      'URL ends with \'leaf-image\'.',
  );
}, 'DOM: Distant leaf (image) satisfies Soft Navigation paint criterion.');
