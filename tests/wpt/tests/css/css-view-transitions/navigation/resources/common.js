function assertViewTransitionOnNavigationImplemented() {
  assert_implements(
      window.CSSViewTransitionRule, "ViewTransition-on-navigation not implemented.");
}

const render_blocking_url = `resources/render-blocking-stylesheet.py`;
let render_block_uuid = null;
let render_block_reject = null;
let render_block_resolve = null;

function renderBlockingOnError() {
  render_block_reject('Error while loading render blocking stylesheet.');
}
function renderBlockingOnLoad() {
  render_block_resolve();
}

function blockRendering() {
  if (document.body)
    throw new Error('Cannot block rendering after body has been parsed.');

  if (render_block_uuid)
    throw new Error('Rendering already blocked.');

  return new Promise((resolve, reject) => {
    render_block_reject = reject;
    render_block_resolve = resolve;
    render_block_uuid = token();
    const href = `${render_blocking_url}?key=${render_block_uuid}`;
    // Need to use document.write since only parser-encountered stylesheets
    // cause render blocking.
    document.write(`<link rel="stylesheet" onerror="renderBlockingOnError()" onload="renderBlockingOnLoad()" href="${href}">`);
    document.close();
  });
}

function unblockRendering() {
  if (!render_block_uuid)
    throw new Error('Rendering not blocked.');

  const url = `${render_blocking_url}?key=${render_block_uuid}`;
  return fetch(url, { method: 'POST' }).then(response => {
    if (response.status != 200) {
      return response.text().then((body) => {
        throw new Error('Failed to unblock rendering: ' + body);
      });
    }
  });
}

// Use external/wpt/html/browsers/browsing-the-web/back-forward-cache/resources/executor.js
// when migrating to external WPTs.
window.disableBFCache = () => {
  return new Promise(resolve => {
    // Use page's UUID as a unique lock name.
    navigator.locks.request("test", () => {
      resolve();
      return new Promise(() => {});
    });
  });
};

function waitForMessage(msg) {
  return new Promise(resolve => {
    window.addEventListener(
      "message", (e) => {
        if (e.data === msg)
          resolve();
        }
  )});
}
