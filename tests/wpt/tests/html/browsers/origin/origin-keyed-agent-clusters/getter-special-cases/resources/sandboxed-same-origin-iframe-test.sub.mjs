import {
  navigateIframe,
  testGetter
} from "../../resources/helpers.mjs";

export default ({ expected }) => {
  // We do this manually instead of using insertIframe because we want to add a
  // sandbox="" attribute and we don't want to set both document.domains.
  promise_setup(() => {
    const iframe = document.createElement("iframe");
    iframe.sandbox = "allow-scripts allow-same-origin";
    const navigatePromise = navigateIframe(iframe, "{{hosts[][]}}", "?1");
    document.body.append(iframe);
    return navigatePromise;
  });

  // Since the allow-same-origin token is set, this should behave like a normal
  // iframe, and follow the embedder.
  testGetter(0, expected);
};
