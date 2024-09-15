'use strict';

// Requires:
// - /common/dispatcher/dispatcher.js
// - /common/utils.js
// - /common/get-host-info.sub.js if automagic conversion of origin names to
// URLs is used.

/**
 * This provides a more friendly interface to remote contexts in dispatches.js.
 * The goal is to make it easy to write multi-window/-frame/-worker tests where
 * the logic is entirely in 1 test file and there is no need to check in any
 * other file (although it is often helpful to check in files of JS helper
 * functions that are shared across remote context).
 *
 * So for example, to test that history traversal works, we create a new window,
 * navigate it to a new document, go back and then go forward.
 *
 * @example
 * promise_test(async t => {
 *   const rcHelper = new RemoteContextHelper();
 *   const rc1 = await rcHelper.addWindow();
 *   const rc2 = await rc1.navigateToNew();
 *   assert_equals(await rc2.executeScript(() => 'here'), 'here', 'rc2 is live');
 *   rc2.historyBack();
 *   assert_equals(await rc1.executeScript(() => 'here'), 'here', 'rc1 is live');
 *   rc1.historyForward();
 *   assert_equals(await rc2.executeScript(() => 'here'), 'here', 'rc2 is live');
 * });
 *
 * Note on the correspondence between remote contexts and
 * `RemoteContextWrapper`s. A remote context is entirely determined by its URL.
 * So navigating away from one and then back again will result in a remote
 * context that can be controlled by the same `RemoteContextWrapper` instance
 * before and after navigation. Messages sent to a remote context while it is
 * destroyed or in BFCache will be queued and processed if that that URL is
 * navigated back to.
 *
 * Navigation:
 * This framework does not keep track of the history of the frame tree and so it
 * is up to the test script to keep track of what remote contexts are currently
 * active and to keep references to the corresponding `RemoteContextWrapper`s.
 *
 * Any action that leads to navigation in the remote context must be executed
 * using
 * @see RemoteContextWrapper.navigate.
 */

{
  const RESOURCES_PATH =
      '/html/browsers/browsing-the-web/remote-context-helper/resources';
  const WINDOW_EXECUTOR_PATH = `${RESOURCES_PATH}/executor-window.py`;
  const WORKER_EXECUTOR_PATH = `${RESOURCES_PATH}/executor-worker.js`;

  /**
   * Turns a string into an origin. If `origin` is null this will return the
   * current document's origin. If `origin` contains not '/', this will attempt
   * to use it as an index in `get_host_info()`. Otherwise returns the input
   * origin.
   * @private
   * @param {string|null} origin The input origin.
   * @return {string|null} The output origin.
   * @throws {RangeError} is `origin` cannot be found in
   *     `get_host_info()`.
   */
  function finalizeOrigin(origin) {
    if (!origin) {
      return location.origin;
    }
    if (!origin.includes('/')) {
      const origins = get_host_info();
      if (origin in origins) {
        return origins[origin];
      } else {
        throw new RangeError(
            `${origin} is not a key in the get_host_info() object`);
      }
    }
    return origin;
  }

  /**
   * @private
   * @param {string} url
   * @returns {string} Absolute url using `location` as the base.
   */
  function makeAbsolute(url) {
    return new URL(url, location).toString();
  }

  async function fetchText(url) {
    return fetch(url).then(r => r.text());
  }

  /**
   * Represents a configuration for a remote context executor.
   */
  class RemoteContextConfig {
    /**
     * @param {Object} [options]
     * @param {string} [options.origin] A URL or a key in `get_host_info()`.
     *                 @see finalizeOrigin for how origins are handled.
     * @param {string[]} [options.scripts]  A list of script URLs. The current
     *     document will be used as the base for relative URLs.
     * @param {[string, string][]} [options.headers]  A list of pairs of name
     *     and value. The executor will be served with these headers set.
     * @param {string} [options.startOn] If supplied, the executor will start
     *     when this event occurs, e.g. "pageshow",
     *     (@see window.addEventListener). This only makes sense for
     *     window-based executors, not worker-based.
     * @param {string} [options.status] If supplied, the executor will pass
     *     this value in the "status" parameter to the executor. The default
     *     executor will default to a status code of 200, if the parameter is
     *     not supplied.
     * @param {string} [options.urlType] Determines what kind of URL is used. Options:
     *     'origin', the URL will be based on the origin;
     *      'data' or 'blob', the URL will contains the document content in
     *      a 'data:' or 'blob:' URL; 'blank', the URL will be blank and the
     *     document content will be written to the initial empty document using
     *     `document.open()`, `document.write()`, and `document.close()`. If not
     *     supplied, the default is 'origin'.
     */
    constructor(
        {origin, scripts = [], headers = [], startOn, status, urlType} = {}) {
      this.origin = origin;
      this.scripts = scripts;
      this.headers = headers;
      this.startOn = startOn;
      this.status = status;
      this.urlType = urlType;
    }

    /**
     * If `config` is not already a `RemoteContextConfig`, one is constructed
     * using `config`.
     * @private
     * @param {object} [config]
     * @returns
     */
    static ensure(config) {
      if (!config) {
        return DEFAULT_CONTEXT_CONFIG;
      }
      return new RemoteContextConfig(config);
    }

    /**
     * Merges `this` with another `RemoteContextConfig` and to give a new
     * `RemoteContextConfig`. `origin` is replaced by the other if present,
     * `headers` and `scripts` are concatenated with `this`'s coming first.
     * @param {RemoteContextConfig} extraConfig
     * @returns {RemoteContextConfig}
     */
    merged(extraConfig) {
      let origin = this.origin;
      if (extraConfig.origin) {
        origin = extraConfig.origin;
      }
      let startOn = this.startOn;
      if (extraConfig.startOn) {
        startOn = extraConfig.startOn;
      }
      let status = this.status;
      if (extraConfig.status) {
        status = extraConfig.status;
      }
      let urlType = this.urlType;
      if (extraConfig.urlType) {
        urlType = extraConfig.urlType;
      }
      const headers = this.headers.concat(extraConfig.headers);
      const scripts = this.scripts.concat(extraConfig.scripts);
      return new RemoteContextConfig(
          {origin, headers, scripts, startOn, status, urlType});
    }

    /**
     * Creates a URL for an executor based on this config.
     * @param {string} uuid The unique ID of the executor.
     * @param {boolean} isWorker If true, the executor will be Worker. If false,
     * it will be a HTML document.
     * @returns {string|Blob|undefined}
     */
    async createExecutorUrl(uuid, isWorker) {
      const origin = finalizeOrigin(this.origin);
      const url = new URL(
          isWorker ? WORKER_EXECUTOR_PATH : WINDOW_EXECUTOR_PATH, origin);

      // UUID is needed for executor.
      url.searchParams.append('uuid', uuid);

      if (this.headers) {
        addHeaders(url, this.headers);
      }
      for (const script of this.scripts) {
        url.searchParams.append('script', makeAbsolute(script));
      }

      if (this.startOn) {
        url.searchParams.append('startOn', this.startOn);
      }

      if (this.status) {
        url.searchParams.append('status', this.status);
      }

      const urlType = this.urlType || 'origin';
      switch (urlType) {
        case 'origin':
        case 'blank':
          return url.href;
        case 'data':
          return `data:text/html;base64,${btoa(await fetchText(url.href))}`;
        case 'blob':
          return URL.createObjectURL(
              new Blob([await fetchText(url.href)], {type: 'text/html'}));
        default:
          throw TypeError(`Invalid urlType: ${urlType}`);
      };
    }
  }

  /**
   * The default `RemoteContextConfig` to use if none is supplied. It has no
   * origin, headers or scripts.
   * @constant {RemoteContextConfig}
   */
  const DEFAULT_CONTEXT_CONFIG = new RemoteContextConfig();

  /**
   * This class represents a configuration for creating remote contexts. This is
   * the entry-point
   * for creating remote contexts, providing @see addWindow .
   */
  class RemoteContextHelper {
    /**
     * @param {RemoteContextConfig|object} config The configuration
     *     for this remote context.
     */
    constructor(config) {
      this.config = RemoteContextConfig.ensure(config);
    }

    /**
     * Creates a new remote context and returns a `RemoteContextWrapper` giving
     * access to it.
     * @private
     * @param {Object} options
     * @param {(url: string) => Promise<undefined>} [options.executorCreator] A
     *     function that takes a URL and causes the browser to navigate some
     *     window to that URL, e.g. via an iframe or a new window. If this is
     *     not supplied, then the returned RemoteContextWrapper won't actually
     *     be communicating with something yet, and something will need to
     *     navigate to it using its `url` property, before communication can be
     *     established.
     * @param {RemoteContextConfig|object} [options.extraConfig] If supplied,
     *     extra configuration for this remote context to be merged with
     *     `this`'s existing config. If it's not a `RemoteContextConfig`, it
     *     will be used to construct a new one.
     * @returns {Promise<RemoteContextWrapper>}
     */
    async createContext({
      executorCreator,
      extraConfig,
      isWorker = false,
    }) {
      const config =
          this.config.merged(RemoteContextConfig.ensure(extraConfig));

      // UUID is needed for executor.
      const uuid = token();
      const url = await config.createExecutorUrl(uuid, isWorker);

      if (executorCreator) {
        if (config.urlType == 'blank') {
          await executorCreator(undefined, await fetchText(url));
        } else {
          await executorCreator(url, undefined);
        }
      }

      return new RemoteContextWrapper(new RemoteContext(uuid), this, url);
    }

    /**
     * Creates a window with a remote context. @see createContext for
     * @param {RemoteContextConfig|object} [extraConfig] Will be
     *     merged with `this`'s config.
     * @param {Object} [options]
     * @param {string} [options.target] Passed to `window.open` as the
     *     2nd argument
     * @param {string} [options.features] Passed to `window.open` as the
     *     3rd argument
     * @returns {Promise<RemoteContextWrapper>}
     */
    addWindow(extraConfig, options) {
      return this.createContext({
        executorCreator: windowExecutorCreator(options),
        extraConfig,
      });
    }
  }
  // Export this class.
  self.RemoteContextHelper = RemoteContextHelper;

  /**
   * Attaches header to the URL. See
   * https://web-platform-tests.org/writing-tests/server-pipes.html#headers
   * @param {string} url the URL to which headers should be attached.
   * @param {[[string, string]]} headers a list of pairs of head-name,
   *     header-value.
   */
  function addHeaders(url, headers) {
    function escape(s) {
      return s.replace('(', '\\(').replace(')', '\\)').replace(',', '\\,');
    }
    const formattedHeaders = headers.map((header) => {
      return `header(${escape(header[0])}, ${escape(header[1])})`;
    });
    url.searchParams.append('pipe', formattedHeaders.join('|'));
  }

  function windowExecutorCreator(
    { target = '_blank', features } = {}, remoteContextWrapper) {
    let openWindow = (url, target, features, documentContent) => {
      const w = window.open(url, target, features);
      if (documentContent) {
        w.document.open();
        w.document.write(documentContent);
        w.document.close();
      }
    };

    return (url, documentContent) => {
      if (url && url.substring(0, 5) == 'data:') {
        throw new TypeError('Windows cannot use data: URLs.');
      }

      if (remoteContextWrapper) {
        return remoteContextWrapper.executeScript(
          openWindow, [url, target, features, documentContent]);
      } else {
        openWindow(url, target, features, documentContent);
      }
    };
  }

  function elementExecutorCreator(
      remoteContextWrapper, elementName, attributes) {
    return (url, documentContent) => {
      return remoteContextWrapper.executeScript(
          (url, elementName, attributes, documentContent) => {
            const el = document.createElement(elementName);
            for (const attribute in attributes) {
              el.setAttribute(attribute, attributes[attribute]);
            }
            if (url) {
              if (elementName == 'object') {
                el.data = url;
              } else {
                el.src = url;
              }
            }
            const parent =
                elementName == 'frame' ? findOrCreateFrameset() : document.body;
            parent.appendChild(el);
            if (documentContent) {
              el.contentDocument.open();
              el.contentDocument.write(documentContent);
              el.contentDocument.close();
            }
          },
          [url, elementName, attributes, documentContent]);
    };
  }

  function iframeSrcdocExecutorCreator(remoteContextWrapper, attributes) {
    return async (url) => {
      // `url` points to the content needed to run an `Executor` in the frame.
      // So we download the content and pass it via the `srcdoc` attribute,
      // setting the iframe's `src` to `undefined`.
      attributes['srcdoc'] = await fetchText(url);

      elementExecutorCreator(
          remoteContextWrapper, 'iframe', attributes)(undefined);
    };
  }

  function workerExecutorCreator(remoteContextWrapper, globalVariable) {
    return url => {
      return remoteContextWrapper.executeScript((url, globalVariable) => {
        const worker = new Worker(url);
        if (globalVariable) {
          window[globalVariable] = worker;
        }
      }, [url, globalVariable]);
    };
  }

  function navigateExecutorCreator(remoteContextWrapper) {
    return url => {
      return remoteContextWrapper.navigate((url) => {
        window.location = url;
      }, [url]);
    };
  }

  /**
   * This class represents a remote context running an executor (a
   * window/frame/worker that can receive commands). It is the interface for
   * scripts to control remote contexts.
   *
   * Instances are returned when new remote contexts are created (e.g.
   * `addFrame` or `navigateToNew`).
   */
  class RemoteContextWrapper {
    /**
     * This should only be constructed by `RemoteContextHelper`.
     * @private
     */
    constructor(context, helper, url) {
      this.context = context;
      this.helper = helper;
      this.url = url;
    }

    /**
     * Executes a script in the remote context.
     * @param {function} fn The script to execute.
     * @param {any[]} args An array of arguments to pass to the script.
     * @returns {Promise<any>} The return value of the script (after
     *     being serialized and deserialized).
     */
    async executeScript(fn, args) {
      return this.context.execute_script(fn, args);
    }

    /**
     * Adds a string of HTML to the executor's document.
     * @param {string} html
     * @returns {Promise<undefined>}
     */
    async addHTML(html) {
      return this.executeScript((htmlSource) => {
        document.body.insertAdjacentHTML('beforebegin', htmlSource);
      }, [html]);
    }

    /**
     * Adds scripts to the executor's document.
     * @param {string[]} urls A list of URLs. URLs are relative to the current
     *     document.
     * @returns {Promise<undefined>}
     */
    async addScripts(urls) {
      if (!urls) {
        return [];
      }
      return this.executeScript(urls => {
        return addScripts(urls);
      }, [urls.map(makeAbsolute)]);
    }

    /**
     * Adds an `iframe` with `src` attribute to the current document.
     * @param {RemoteContextConfig} [extraConfig]
     * @param {[string, string][]} [attributes] A list of pairs of strings
     *     of attribute name and value these will be set on the iframe element
     *     when added to the document.
     * @returns {Promise<RemoteContextWrapper>} The remote context.
     */
    addIframe(extraConfig, attributes = {}) {
      return this.helper.createContext({
        executorCreator: elementExecutorCreator(this, 'iframe', attributes),
        extraConfig,
      });
    }

    /**
     * Adds a `frame` with `src` attribute to the current document's first
     * `frameset` element.
     * @param {RemoteContextConfig} [extraConfig]
     * @param {[string, string][]} [attributes] A list of pairs of strings
     *     of attribute name and value these will be set on the frame element
     *     when added to the document.
     * @returns {Promise<RemoteContextWrapper>} The remote context.
     */
    addFrame(extraConfig, attributes = {}) {
      return this.helper.createContext({
        executorCreator: elementExecutorCreator(this, 'frame', attributes),
        extraConfig,
      });
    }

    /**
     * Adds an `embed` with `src` attribute to the current document.
     * @param {RemoteContextConfig} [extraConfig]
     * @param {[string, string][]} [attributes] A list of pairs of strings
     *     of attribute name and value these will be set on the embed element
     *     when added to the document.
     * @returns {Promise<RemoteContextWrapper>} The remote context.
     */
    addEmbed(extraConfig, attributes = {}) {
      return this.helper.createContext({
        executorCreator: elementExecutorCreator(this, 'embed', attributes),
        extraConfig,
      });
    }

    /**
     * Adds an `object` with `data` attribute to the current document.
     * @param {RemoteContextConfig} [extraConfig]
     * @param {[string, string][]} [attributes] A list of pairs of strings
     *     of attribute name and value these will be set on the object element
     *     when added to the document.
     * @returns {Promise<RemoteContextWrapper>} The remote context.
     */
    addObject(extraConfig, attributes = {}) {
      return this.helper.createContext({
        executorCreator: elementExecutorCreator(this, 'object', attributes),
        extraConfig,
      });
    }

    /**
     * Adds an iframe with `srcdoc` attribute to the current document
     * @param {RemoteContextConfig} [extraConfig]
     * @param {[string, string][]} [attributes] A list of pairs of strings
     *     of attribute name and value these will be set on the iframe element
     *     when added to the document.
     * @returns {Promise<RemoteContextWrapper>} The remote context.
     */
    addIframeSrcdoc(extraConfig, attributes = {}) {
      return this.helper.createContext({
        executorCreator: iframeSrcdocExecutorCreator(this, attributes),
        extraConfig,
      });
    }

    /**
     * Opens a window from the remote context. @see createContext for
     * @param {RemoteContextConfig|object} [extraConfig]
     * @param {Object} [options]
     * @param {string} [options.target] Passed to `window.open` as the
     *     2nd argument
     * @param {string} [options.features] Passed to `window.open` as the
     *     3rd argument
     * @returns {Promise<RemoteContextWrapper>} The remote context.
     */
    addWindow(extraConfig, options) {
      return this.helper.createContext({
        executorCreator: windowExecutorCreator(options, this),
        extraConfig,
      });
    }

    /**
     * Adds a dedicated worker to the current document.
     * @param {string|null} [globalVariable] The name of the global variable to
     *   which to assign the `Worker` object after construction. If `null`,
     *   then no assignment will take place.
     * @param {RemoteContextConfig} [extraConfig]
     * @returns {Promise<RemoteContextWrapper>} The remote context.
     */
    addWorker(globalVariable, extraConfig) {
      return this.helper.createContext({
        executorCreator: workerExecutorCreator(this, globalVariable),
        extraConfig,
        isWorker: true,
      });
    }

    /**
     * Gets a `Headers` object containing the request headers that were used
     * when the browser requested this document.
     *
     * Currently, this only works for `RemoteContextHelper`s representing
     * windows, not workers.
     * @returns {Promise<Headers>}
     */
    async getRequestHeaders() {
      // This only works in window environments for now. We could make it work
      // for workers too; if you have a need, just share or duplicate the code
      // that's in executor-window.py. Anyway, we explicitly use `window` in
      // the script so that we get a clear error if you try using it on a
      // worker.

      // We need to serialize and deserialize the `Headers` object manually.
      const asNestedArrays = await this.executeScript(() => [...window.__requestHeaders]);
      return new Headers(asNestedArrays);
    }

    /**
     * Executes a script in the remote context that will perform a navigation.
     * To do this safely, we must suspend the executor and wait for that to
     * complete before executing. This ensures that all outstanding requests are
     * completed and no more can start. It also ensures that the executor will
     * restart if the page goes into BFCache or it was a same-document
     * navigation. It does not return a value.
     *
     * NOTE: We cannot monitor whether and what navigations are happening. The
     * logic has been made as robust as possible but is not fool-proof.
     *
     * Foolproof rule:
     * - The script must perform exactly one navigation.
     * - If that navigation is a same-document history traversal, you must
     * `await` the result of `waitUntilLocationIs`. (Same-document non-traversal
     * navigations do not need this extra step.)
     *
     * More complex rules:
     * - The script must perform a navigation. If it performs no navigation,
     *   the remote context will be left in the suspended state.
     * - If the script performs a direct same-document navigation, it is not
     * necessary to use this function but it will work as long as it is the only
     *   navigation performed.
     * - If the script performs a same-document history navigation, you must
     * `await` the result of `waitUntilLocationIs`.
     *
     * @param {function} fn The script to execute.
     * @param {any[]} args An array of arguments to pass to the script.
     * @returns {Promise<undefined>}
     */
    navigate(fn, args) {
      return this.executeScript((fnText, args) => {
        executeScriptToNavigate(fnText, args);
      }, [fn.toString(), args]);
    }

    /**
     * Navigates to the given URL, by executing a script in the remote
     * context that will perform navigation with the `location.href`
     * setter.
     *
     * Be aware that performing a cross-document navigation using this
     * method will cause this `RemoteContextWrapper` to become dormant,
     * since the remote context it points to is no longer active and
     * able to receive messages. You also won't be able to reliably
     * tell when the navigation finishes; the returned promise will
     * fulfill when the script finishes running, not when the navigation
     * is done. As such, this is most useful for testing things like
     * unload behavior (where it doesn't matter) or prerendering (where
     * there is already a `RemoteContextWrapper` for the destination).
     * For other cases, using `navigateToNew()` will likely be better.
     *
     * @param {string|URL} url The URL to navigate to.
     * @returns {Promise<undefined>}
     */
    navigateTo(url) {
      return this.navigate(url => {
        location.href = url;
      }, [url.toString()]);
    }

    /**
     * Navigates the context to a new document running an executor.
     * @param {RemoteContextConfig} [extraConfig]
     * @returns {Promise<RemoteContextWrapper>} The remote context.
     */
    async navigateToNew(extraConfig) {
      return this.helper.createContext({
        executorCreator: navigateExecutorCreator(this),
        extraConfig,
      });
    }

    //////////////////////////////////////
    // Navigation Helpers.
    //
    // It is up to the test script to know which remote context will be
    // navigated to and which `RemoteContextWrapper` should be used after
    // navigation.
    //
    // NOTE: For a same-document history navigation, the caller use `await` a
    // call to `waitUntilLocationIs` in order to know that the navigation has
    // completed. For convenience the method below can return the promise to
    // wait on, if passed the expected location.

    async waitUntilLocationIs(expectedLocation) {
      return this.executeScript(async (expectedLocation) => {
        if (location.href === expectedLocation) {
          return;
        }

        // Wait until the location updates to the expected one.
        await new Promise(resolve => {
          const listener = addEventListener('hashchange', (event) => {
            if (event.newURL === expectedLocation) {
              removeEventListener(listener);
              resolve();
            }
          });
        });
      }, [expectedLocation]);
    }

    /**
     * Performs a history traversal.
     * @param {integer} n How many steps to traverse. @see history.go
     * @param {string} [expectedLocation] If supplied will be passed to @see waitUntilLocationIs.
     * @returns {Promise<undefined>}
     */
    async historyGo(n, expectedLocation) {
      await this.navigate((n) => {
        history.go(n);
      }, [n]);
      if (expectedLocation) {
        await this.waitUntilLocationIs(expectedLocation);
      }
    }

    /**
     * Performs a history traversal back.
     * @param {string} [expectedLocation] If supplied will be passed to @see waitUntilLocationIs.
     * @returns {Promise<undefined>}
     */
    async historyBack(expectedLocation) {
      await this.navigate(() => {
        history.back();
      });
      if (expectedLocation) {
        await this.waitUntilLocationIs(expectedLocation);
      }
    }

    /**
     * Performs a history traversal back.
     * @param {string} [expectedLocation] If supplied will be passed to @see waitUntilLocationIs.
     * @returns {Promise<undefined>}
     */
    async historyForward(expectedLocation) {
      await this.navigate(() => {
        history.forward();
      });
      if (expectedLocation) {
        await this.waitUntilLocationIs(expectedLocation);
      }
    }
  }
}
