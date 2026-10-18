// Helpers for the focusreset event tests.
// https://github.com/whatwg/html/pull/13050

// focusreset is fired from the mutation observer microtask (after
// MutationObserver callbacks and slotchange events) that is queued when the
// focused element is removed.

// Resolves in a new task, i.e. after all pending microtasks (including the one
// that fires focusreset) have run.
function nextTask(t) {
  return new Promise(resolve => t.step_timeout(resolve, 0));
}

// Resolves after the next "update the rendering" has fully completed.
function afterNextRenderingUpdate() {
  return new Promise(resolve => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
  });
}

// Resolves after the next "update the rendering" and an additional task, to
// give a chance for anything that was incorrectly deferred to run.
async function afterNextRenderingUpdateAndTask(t) {
  await afterNextRenderingUpdate();
  await nextTask(t);
}

// Makes sure the test isn't running from inside a rendering update (e.g. a
// ResizeObserver callback) and that the document is being rendered.
function settle(t) {
  return new Promise(resolve => {
    requestAnimationFrame(() => t.step_timeout(resolve, 0));
  });
}

// Records focusreset events dispatched at |target| for the duration of the
// test |t|. Returns the array that the events are pushed into.
function recordFocusResets(t, target = document) {
  const events = [];
  const listener = e => events.push(e);
  target.addEventListener('focusreset', listener);
  t.add_cleanup(() => target.removeEventListener('focusreset', listener));
  return events;
}

// Records events of the given types dispatched at |target| (capturing, so that
// non-bubbling events on descendants are seen as well).
function recordEvents(t, target, types) {
  const events = [];
  const listener = e => events.push(e.type);
  for (const type of types) {
    target.addEventListener(type, listener, true);
  }
  t.add_cleanup(() => {
    for (const type of types) {
      target.removeEventListener(type, listener, true);
    }
  });
  return events;
}

// Creates an element that is removed when the test |t| finishes.
function createFocusable(t, tagName = 'button', parent = document.body) {
  const el = document.createElement(tagName);
  el.textContent = tagName;
  parent.append(el);
  t.add_cleanup(() => el.remove());
  return el;
}
