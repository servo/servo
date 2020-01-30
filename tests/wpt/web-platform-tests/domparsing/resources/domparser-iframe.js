window.doParse = (html, mimeType) => {
  const parser = new DOMParser();
  return parser.parseFromString(html, mimeType);
};
