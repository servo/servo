function determineInjectionSinkDescription(testCase) {
    const targetWindowDescription = ("targetWindow" in testCase) ?
      testCase.targetWindow.name : "";

    const element = ("elementId" in testCase) ?
      window.document.getElementById(testCase.elementId) : null;

    const elementDescription = element ? (element.localName +
        (element.target ? ("[target=" + element.target + "]") : "")) : null;

    return ((elementDescription ? (elementDescription + ".") :
      (targetWindowDescription ? (targetWindowDescription + ".") : ""))) +
      testCase.propertySequence.join(".");
}

function assignJavascriptURLToInjectionSink(testCase) {
  const element = ("elementId" in testCase) ?
    document.getElementById(testCase.elementId) : null;

  let currentObject = element ? element : testCase.targetWindow;

  const propertySequence = testCase.propertySequence;
  for (let i = 0; i < propertySequence.length - 1; ++i) {
    currentObject = currentObject[propertySequence[i]];
  }

  currentObject[propertySequence.at(-1)] =
    "javascript:parent.postMessage('executed', '*')";

  if ("navigationFunction" in testCase) {
    element[testCase.navigationFunction]();
  }
}

function encodeURIWithApostrophes(uriWithApostrophes) {
  const encodedURI = encodeURI(uriWithApostrophes);
  // https://developer.mozilla.org/en-US/docs/Glossary/Percent-encoding
  return encodedURI.replaceAll("'","%27");
}

function openWindowAssignJavascriptUrlEvaluatedToIframeStringAndCheckViolatedDirective(
  windowOpenArguments, reportValue) {
  const win = window.open(windowOpenArguments.url, windowOpenArguments.target,
    windowOpenArguments.windowFeatures);

  win.addEventListener("load", () => {
    // <frame-wth-csp.sub.html> has no CSP if no "?csp=..." argument is passed.
    win.location.href =
      "javascript:'<iframe src=support/frame-with-csp.sub.html></iframe>'";

    const checkReportScript = document.createElement("script");
    checkReportScript.src = encodeURIWithApostrophes(
      "../support/checkReport.sub.js?reportField=violated-directive&reportValue=" + reportValue);
    document.body.appendChild(checkReportScript);
  });
}
