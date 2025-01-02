import {
  navigateIframe,
  testGetter
} from "../../resources/helpers.mjs";

export default () => {
  // We do this manually instead of using insertIframe because we want to add a
  // sandbox="" attribute and we don't want to set both document.domains.
  promise_setup(() => {
    const iframe = document.createElement("iframe");
    iframe.sandbox = "allow-scripts";
    const navigatePromise = navigateIframe(iframe, "{{hosts[][]}}", "?1");
    document.body.append(iframe);
    return navigatePromise;
  });

  // Sandboxed iframes have an opaque origin, so it should return true, since
  // for them site === origin so they are always origin-keyed.
  testGetter(0, true);
};
