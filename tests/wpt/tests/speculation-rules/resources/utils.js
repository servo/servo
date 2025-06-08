globalThis.assertSpeculationRulesIsSupported = () => {
  assert_implements(
      'supports' in HTMLScriptElement,
      'HTMLScriptElement.supports must be supported');
  assert_implements(
      HTMLScriptElement.supports('speculationrules'),
      '<script type="speculationrules"> must be supported');
};

// If you want access to these, be sure to include
// /html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js.
// So as to avoid requiring everyone to do that, we only conditionally define this infrastructure.
if (globalThis.RemoteContextHelper) {
  class PreloadingRemoteContextWrapper extends RemoteContextHelper.RemoteContextWrapper {
    /**
    * Starts preloading a page with this `PreloadingRemoteContextWrapper` as the
    * referrer, using `<script type="speculationrules">`.
    *
    * @param {"prefetch"|"prerender"} preloadType - The preload type to use.
    * @param {object} [extrasInSpeculationRule] - Additional properties to add
    *     to the speculation rule JSON.
    * @param {RemoteContextConfig|object} [extraConfig] - Additional remote
    *     context configuration for the preloaded context.
    * @returns {Promise<PreloadingRemoteContextWrapper>}
    */
    addPreload(preloadType, { extrasInSpeculationRule = {}, ...extraConfig } = {}) {
      const referrerRemoteContext = this;

      return this.helper.createContext({
        executorCreator(url) {
          return referrerRemoteContext.executeScript((url, preloadType, extrasInSpeculationRule) => {
            const script = document.createElement("script");
            script.type = "speculationrules";
            script.textContent = JSON.stringify({
              [preloadType]: [
                {
                  source: "list",
                  urls: [url],
                  ...extrasInSpeculationRule
                }
              ]
            });
            document.head.append(script);
          }, [url, preloadType, extrasInSpeculationRule]);
        },
        extraConfig
      });
    }
  }

  globalThis.PreloadingRemoteContextHelper = class extends RemoteContextHelper {
    static RemoteContextWrapper = PreloadingRemoteContextWrapper;
  };
}
