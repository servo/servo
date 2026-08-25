// Used by the served (src=) cases. The srcdoc/data:/blob: cases in
// get-opaque-origin-combinations.https.html inline the same logic because they can
// neither load an external page nor carry an id in a URL.
(async () => {
  const id = new URL(location.href).searchParams.get("id");
  const report = {
    id,
    origin: String(self.origin),
    secure: self.isSecureContext,
    result: "resolved",
  };
  try {
    if (!(navigator.credentials && navigator.credentials.get))
      throw new TypeError("navigator.credentials.get is unavailable");
    await navigator.credentials.get({ digital: { requests: [] } });
  } catch (error) {
    report.result = error.name;
  }
  (window.top || window.parent).postMessage(report, "*");
})();
