const loadPromise = new Promise(resolve => { window.resolveLoadPromise = resolve; });

function assertURL(doc, expectedURL) {
  assert_equals(doc.URL, expectedURL, "document.URL");
  assert_equals(doc.documentURI, expectedURL, "document.documentURI");
  assert_equals(doc.baseURI, expectedURL, "document.baseURI");
}

const supportedTypes = [
  "text/html",
  "text/xml",
  "application/xml",
  "application/xhtml+xml",
  "image/svg+xml",
];

const invalidXML = `<span x:test="testing">1</span>`;
const inputs = {
  valid: "<html></html>",
  "invalid XML": invalidXML
};

for (const mimeType of supportedTypes) {
  for (const [inputName, input] of Object.entries(inputs)) {
    if (mimeType === "text/html" && input === invalidXML) {
      continue;
    }

    test(() => {
      const parser = new DOMParser();
      const doc = parser.parseFromString(input, mimeType);

      assertURL(doc, document.URL);
    }, `${mimeType} ${inputName}: created normally`);

    promise_test(async () => {
      await loadPromise;

      const parser = new frames[0].DOMParser();
      const doc = parser.parseFromString(input, mimeType);

      assertURL(doc, frames[0].document.URL);
    }, `${mimeType} ${inputName}: created using another iframe's DOMParser from this frame`);

    promise_test(async () => {
      await loadPromise;

      const parser = new frames[0].DOMParser();
      const doc = frames[0].doParse(input, mimeType);

      assertURL(doc, frames[0].document.URL);
    }, `${mimeType} ${inputName}: created using another iframe's DOMParser from that frame`);

    promise_test(async () => {
      await loadPromise;

      const parser = new DOMParser();
      const doc = frames[0].DOMParser.prototype.parseFromString.call(parser, input, mimeType);

      assertURL(doc, document.URL);
    }, `${mimeType} ${inputName}: created using a parser from this frame and the method from the iframe`);

    promise_test(async () => {
      await loadPromise;

      const parser = new frames[0].DOMParser();
      const doc = DOMParser.prototype.parseFromString.call(parser, input, mimeType);

      assertURL(doc, frames[0].document.URL);
    }, `${mimeType} ${inputName}: created using a parser from the iframe and the method from this frame`);
  }
}
