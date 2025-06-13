// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../resources/soft-navigation-test-helper.js

// This test is very similar to distant-leaf-text.window.js, but the leaf
// node is text instead of an image.
//
// This test is intended to verify that when a distant leaf of a
// deeply nested div element is attached to the DOM, its painting can
// trigger a soft navigation.
//
// To show this, we create a button that, when clicked, creates a deeply
// nested div element and attaches it to the DOM - only the leaf, a text
// node saying "Hello, World.", 10 levels below the attachment point actually
// gets painted.

function clickHandler() {
  let div = document.createElement('div');
  div.textContent = 'Hello, World.';  // The leaf node that gets painted.
  for (let i = 0; i < 10; i++) {
    const tmp = document.createElement('div');
    tmp.appendChild(div);
    div = tmp;
  }
  document.body.appendChild(div);
  history.pushState({}, '', '/greeting');
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
      /*includeSoftNavigationObservations=*/ false,
      /*minNumEntries=*/ 1,
      /*timeout=*/ 3000
    );
  assert_equals(entries.length, 1, 'Expected exactly one soft navigation.');
  assert_equals(
      entries[0].name.replace(/.*\//, ''),
      'greeting',
      'URL ends with \'greeting\'.',
  );
}, 'DOM: Distant leaf (text) satisfies Soft Navigation paint criterion.');
