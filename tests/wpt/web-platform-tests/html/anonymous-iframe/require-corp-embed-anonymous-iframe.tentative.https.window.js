// META: script=/common/utils.js

promise_test(async t => {
  let iframe_allowed = (iframe) => new Promise(async resolve => {
    window.addEventListener("message", t.step_func(msg => {
      if (msg.source !== iframe.contentWindow) return;
      assert_equals(msg.data, "loaded",
                    "Unexpected message from broadcast channel.");
      resolve(true);
    }));

    // To see whether the iframe was blocked, we check whether it
    // becomes cross-origin (since error pages are loaded cross-origin).
    await t.step_wait(() => {
      try {
        // Accessing contentWindow.location.href cross-origin throws.
        iframe.contentWindow.location.href === null;
        return false;
      } catch {
        return true;
      }
    });
    resolve(false);
  });

  // Create an anonymous child iframe.
  const child = document.createElement("iframe");
  child.anonymous = true;
  t.add_cleanup(() => child.remove());

  child.src = "/html/cross-origin-embedder-policy/resources/" +
    "navigate-none.sub.html?postMessageTo=top";
  document.body.append(child);

  assert_true(await iframe_allowed(child),
              "The anonymous iframe should be allowed.");

  // Create a child of the anonymous iframe. Even if the grandchild
  // does not have the 'anonymous' attribute set, it inherits the
  // anonymous property from the parent.
  const grandchild = child.contentDocument.createElement("iframe");

  grandchild.src = "/html/cross-origin-embedder-policy/resources/" +
    "navigate-none.sub.html?postMessageTo=top";
  child.contentDocument.body.append(grandchild);

  assert_true(await iframe_allowed(grandchild),
             "The child of the anonymous iframe should be allowed.");
}, 'Loading an anonymous iframe with COEP: require-corp is allowed.');
