function elDesc(el) {
  let rv = `<${el.localName}`;
  if (el.hasAttribute('contenteditable')) {
    rv += ` contenteditable="${el.getAttribute('contenteditable')}"`;
  }
  if (el.hasAttribute('type')) {
    rv += ` type="${el.getAttribute('type')}"`;
  }
  rv += `>`;
  return rv;
}

function setSelection(el, selectionStart, selectionEnd) {
  if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement) {
    el.selectionStart = selectionStart;
    el.selectionEnd = selectionEnd;
  } else {
    const s = getSelection();
    s.removeAllRanges();
    const r = new Range();
    r.setStart(el.firstChild || el, selectionStart);
    r.setEnd(el.firstChild || el, selectionEnd);
    s.addRange(r);
  }
}

function getValue(el) {
  if (el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement) {
    return el.value;
  }
  return el.innerHTML;
}

const keyMapping = {
  "Enter": "\uE006",
  "Backspace": "\uE003",
  "Delete": "\uE017",
};
