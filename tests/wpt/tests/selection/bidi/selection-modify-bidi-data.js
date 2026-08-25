// Shared test data and fixtures for Selection.modify() character movement
// in bidi text. Used by modify-move-by-character.html and
// modify-extend-by-character.html.

function createBidiFixtures() {
  const container = document.createElement("div");
  container.id = "container";
  container.style.cssText = "font: 16px monospace";
  // Visual rendering on screen (left-to-right):
  // pure-ltr:    [Hello World           ]
  // pure-rtl:    [           مرحبا عالم]
  // rtl-in-ltr:  [אבגדהו                ]
  // ltr-in-rtl:  [           Hello World]
  // mixed-ltr:   [Hello אבגדהו World    ]
  // mixed-auto:  [    Hello World אבגדהו]
  container.innerHTML =
    '<div id="pure-ltr" dir="ltr">Hello World</div>' +
    '<div id="pure-rtl" dir="rtl">\u0645\u0631\u062D\u0628\u0627 \u0639\u0627\u0644\u0645</div>' +
    '<div id="rtl-in-ltr" dir="ltr">\u05D0\u05D1\u05D2\u05D3\u05D4\u05D5</div>' +
    '<div id="ltr-in-rtl" dir="rtl">Hello World</div>' +
    '<div id="mixed-ltr" dir="ltr">Hello \u05D0\u05D1\u05D2\u05D3\u05D4\u05D5 World</div>' +
    '<div id="mixed-auto" dir="auto">\u05D0\u05D1\u05D2\u05D3\u05D4\u05D5 Hello World</div>';
  document.body.appendChild(container);
}

// Each entry: [divId, count, direction, startOffset, endOffset, name]
// The name uses {alter} as a placeholder for "move" or "extend".

const logicalTests = [
  ["pure-ltr", 2, "forward", 3, 5,
    "LTR text in LTR paragraph: {alter} forward 2 chars"],
  ["pure-ltr", 2, "backward", 5, 3,
    "LTR text in LTR paragraph: {alter} backward 2 chars"],

  ["pure-rtl", 2, "forward", 2, 4,
    "RTL text in RTL paragraph: {alter} forward 2 chars"],
  ["pure-rtl", 2, "backward", 4, 2,
    "RTL text in RTL paragraph: {alter} backward 2 chars"],

  ["rtl-in-ltr", 2, "forward", 2, 4,
    "RTL text in LTR paragraph: {alter} forward 2 chars"],
  ["rtl-in-ltr", 2, "backward", 4, 2,
    "RTL text in LTR paragraph: {alter} backward 2 chars"],

  ["ltr-in-rtl", 2, "forward", 3, 5,
    "LTR text in RTL paragraph: {alter} forward 2 chars"],
  ["ltr-in-rtl", 2, "backward", 5, 3,
    "LTR text in RTL paragraph: {alter} backward 2 chars"],

  ["mixed-ltr", 3, "forward", 4, 7,
    "LTR-RTL context in LTR paragraph: {alter} forward 3 chars"],
  ["mixed-ltr", 3, "backward", 11, 8,
    "LTR-RTL context in LTR paragraph: {alter} backward 3 chars"],
  ["mixed-ltr", 3, "forward", 7, 10,
    "RTL-LTR context in LTR paragraph: {alter} forward 3 chars"],
  ["mixed-ltr", 3, "backward", 14, 11,
    "RTL-LTR context in LTR paragraph: {alter} backward 3 chars"],

  ["mixed-auto", 3, "forward", 5, 8,
    "Mixed context in auto-dir paragraph: {alter} forward 3 chars"],
  ["mixed-auto", 3, "backward", 17, 14,
    "Mixed context in auto-dir paragraph: {alter} backward 3 chars"],
];

// Left/right uses the resolved text direction at the focus, not the
// paragraph's inline base direction.
// See https://github.com/w3c/selection-api/pull/357
//
// In mixed bidi text, extend moves the focus in the correct visual direction,
// but the selection range is still a single logical (anchor, focus) pair. At
// bidi boundaries the visually highlighted region may appear non-contiguous
// because the logical range does not map to a contiguous visual span.

const visualTests = [
  ["pure-ltr", 2, "right", 3, 5,
    "LTR text in LTR paragraph: {alter} right 2 chars"],
  ["pure-ltr", 2, "left", 5, 3,
    "LTR text in LTR paragraph: {alter} left 2 chars"],

  ["pure-rtl", 2, "right", 4, 2,
    "RTL text in RTL paragraph: {alter} right 2 chars"],
  ["pure-rtl", 2, "left", 2, 4,
    "RTL text in RTL paragraph: {alter} left 2 chars"],

  ["rtl-in-ltr", 2, "right", 4, 2,
    "RTL text in LTR paragraph: {alter} right 2 chars"],
  ["rtl-in-ltr", 2, "left", 2, 4,
    "RTL text in LTR paragraph: {alter} left 2 chars"],

  ["ltr-in-rtl", 2, "right", 3, 5,
    "LTR text in RTL paragraph: {alter} right 2 chars"],
  ["ltr-in-rtl", 2, "left", 5, 3,
    "LTR text in RTL paragraph: {alter} left 2 chars"],

  ["mixed-ltr", 3, "right", 4, 11,
    "LTR-RTL context in LTR paragraph: {alter} right 3 chars"],
  ["mixed-ltr", 3, "left", 11, 4,
    "LTR-RTL context in LTR paragraph: {alter} left 3 chars"],
  ["mixed-ltr", 3, "right", 7, 14,
    "RTL-LTR context in LTR paragraph: {alter} right 3 chars"],
  ["mixed-ltr", 3, "left", 14, 7,
    "RTL-LTR context in LTR paragraph: {alter} left 3 chars"],

  ["mixed-auto", 3, "right", 17, 5,
    "Mixed context in auto-dir paragraph: {alter} right 3 chars"],
  ["mixed-auto", 3, "left", 5, 17,
    "Mixed context in auto-dir paragraph: {alter} left 3 chars"],
];
