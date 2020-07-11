'use strict';

/* Whether the browser is Chromium-based with MojoJS enabled */
const isChromiumBased = 'MojoInterfaceInterceptor' in self;
/* Whether the browser is WebKit-based with internal test-only API enabled */
const isWebKitBased = !isChromiumBased && 'internals' in self;

/**
 * Loads a script in a window or worker.
 *
 * @param {string} path - A script path
 * @returns {Promise}
 */
function loadScript(path) {
  if (typeof document === 'undefined') {
    // Workers (importScripts is synchronous and may throw.)
    importScripts(path);
    return Promise.resolve();
  } else {
    // Window
    const script = document.createElement('script');
    script.src = path;
    script.async = false;
    const p = new Promise((resolve, reject) => {
      script.onload = () => { resolve(); };
      script.onerror = e => { reject(e); };
    })
    document.head.appendChild(script);
    return p;
  }
}

/**
 * A helper for Chromium-based browsers to load Mojo JS bindingds
 *
 * This is an async function that works in both workers and windows. It first
 * loads mojo_bindings.js, disables automatic dependency loading, and loads all
 * resources sequentially. The promise resolves if everything loads
 * successfully, or rejects if any exception is raised. If testharness.js is
 * used, an uncaught exception will terminate the test with a harness error
 * (unless `allow_uncaught_exception` is true), which is usually the desired
 * behaviour. Only call this function if isChromiumBased === true.
 *
 * @param {Array.<string>} resources - A list of scripts to load: Mojo JS
 *   bindings should be of the form '/gen/../*.mojom.js', the ordering of which
 *   does not matter. Do not include mojo_bindings.js in this list. You may
 *   include other non-mojom.js scripts for convenience.
 * @returns {Promise}
 */
async function loadMojoResources(resources) {
  if (!isChromiumBased) {
    throw new Error('MojoJS not enabled; start Chrome with --enable-blink-features=MojoJS,MojoJSTest');
  }
  if (resources.length == 0) {
    return;
  }

  // We want to load mojo_bindings.js separately to set mojo.config.
  if (resources.some(p => p.endsWith('/mojo_bindings.js'))) {
    throw new Error('Do not load mojo_bindings.js explicitly.');
  }
  await loadScript('/gen/layout_test_data/mojo/public/js/mojo_bindings.js');
  mojo.config.autoLoadMojomDeps = false;

  for (const path of resources) {
    await loadScript(path);
  }
}
