'use strict';

function RunTestsInIFrame(sourceURL) {
  let frame = document.createElement('iframe');
  frame.src = sourceURL;
  document.body.appendChild(frame);
  fetch_tests_from_window(frame.contentWindow);
}

function RunTestsInNestedIFrame(sourceURL) {
  let nestedFrame = document.createElement('iframe');
  document.body.appendChild(nestedFrame);
  let content = `
    <script src="/resources/testharness.js"></script>
    <script src="helpers.js"></script>
    <body>
    <script>
      RunTestsInIFrame("${sourceURL}");
    </sc` + `ript>
  `;

    nestedFrame.contentDocument.write(content);
    nestedFrame.contentDocument.close();
    fetch_tests_from_window(nestedFrame.contentWindow);
}