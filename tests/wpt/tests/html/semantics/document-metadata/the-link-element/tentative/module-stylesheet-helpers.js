// Shared helpers for `<link rel="stylesheet" import>` tentative tests.

export function createLink(t, specifier, href) {
  const link = document.createElement('link');
  t.add_cleanup(() => link.remove());
  link.rel = 'stylesheet';
  if (specifier !== undefined) {
    link.setAttribute('import', specifier);
  }
  if (href !== undefined) {
    link.href = href;
  }
  return link;
}

export function loadResult(t, specifier, href) {
  return appendAndWait(createLink(t, /*import=*/ specifier, /*href=*/ href));
}

export function waitForEvent(link) {
  return new Promise((resolve) => {
    link.addEventListener('load', () => resolve('load'), {once: true});
    link.addEventListener('error', () => resolve('error'), {once: true});
  });
}

export function appendAndWait(link) {
  const result = waitForEvent(link);
  document.head.appendChild(link);
  return result.then(event => ({link, event}));
}

// Creates a shadow host + open root and returns the root.
export function createShadowRoot(t, markup = '') {
  const host = document.createElement('div');
  document.body.append(host);
  if (t && t.add_cleanup) {
    t.add_cleanup(() => host.remove());
  }
  const root = host.attachShadow({mode: 'open'});
  root.innerHTML = markup;
  return root;
}

// Creates a document-scoped container removed after the test.
export function createDocumentContainer(t, markup = '') {
  const container = document.createElement('div');
  container.innerHTML = markup;
  document.body.append(container);
  t.add_cleanup(() => container.remove());
  return container;
}

// Count of occurrences of `sheet` in `scope.styleSheets`.
export function styleSheetsCount(scope, sheet) {
  return Array.from(scope.styleSheets).filter(s => s === sheet).length;
}

// Requires /common/utils.js. Each resource has independent server state and
// bypasses the HTTP cache so counts measure module-map reuse.
export function controlledModule(t) {
  const url =
      new URL('module-stylesheet-controlled.py?key=' + token(), location.href)
          .href;
  async function command(action, options = '') {
    const response = await fetch(url + '&action=' + action + options);
    assert_true(response.ok);
    return response.json();
  }
  t.add_cleanup(() => command('release'));
  return {
    url,
    status: () => command('status'),
    release: () => command('release'),
    configure: (status, type = 'text/css') =>
        command('configure', '&status=' + status + '&type=' + type),
    async started() {
      while ((await command('status')).count === 0) {
        await new Promise(resolve => t.step_timeout(resolve, 0));
      }
    },
  };
}

export function recordLinkEvents(link) {
  const events = [];
  for (const name of ['load', 'error']) {
    link.addEventListener(name, () => events.push(name));
  }
  return events;
}
