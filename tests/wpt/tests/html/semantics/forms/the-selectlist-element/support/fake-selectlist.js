function replaceChildElement(newChild, oldChild) {
  oldChild.parentElement.replaceChild(newChild, oldChild);
}

function createFakeSelectlist(selectedValueText, includeListbox) {
  const selectlist = document.createElement('div');
  selectlist.classList.add('fake-selectlist');
  selectlist.innerHTML = `
    <button class="fake-selectlist-internal-selectlist-button">
      <div class="fake-selectlist-selected-value"></div>
      <div class="fake-selectlist-internal-selectlist-button-icon"></div>
    </button>
  `;
  if (includeListbox) {
    const listbox = document.createElement('div');
    listbox.classList.add('fake-selectlist-listbox');
    listbox.anchorElement = selectlist;
    selectlist.appendChild(listbox);
  }
  selectlist.appendChild(createFakeSelectlistStyles());

  const selectedvalue = selectlist.querySelector('.fake-selectlist-selected-value');
  if (selectedValueText) {
    selectedvalue.textContent = selectedValueText;
  }

  return selectlist;
}

function createFakeSelectlistStyles() {
  const style = document.createElement('style');
  style.textContent = `
    .fake-selectlist {
      display: inline-flex;
      font-family: sans-serif;
      font-size: 0.875em;
      user-select: none;
    }

    .fake-selectlist-internal-selectlist-button {
      color: fieldtext;
      background-color: field;
      appearance: none;
      cursor: default;
      font-size: inherit;
      text-align: inherit;
      display: inline-flex;
      flex-grow: 1;
      flex-shrink: 1;
      align-items: center;
      overflow-x: hidden;
      overflow-y: hidden;
      padding: 0.25em;
      border-width: 1px;
      border-style: solid;
      border-color: buttonborder;
      border-image: initial;
      border-radius: 0.25em;
    }

    .fake-selectlist-selected-value {
      color: FieldText;
      flex-grow:1;
    }

    .fake-selectlist-internal-selectlist-button-icon {
      background-image: url(support/selectlist_button_icon.svg);
      background-origin: content-box;
      background-repeat: no-repeat;
      background-size: contain;
      height: 1.0em;
      margin-inline-start: 4px;
      opacity: 1;
      outline: none;
      padding-bottom: 2px;
      padding-inline-start: 3px;
      padding-inline-end: 3px;
      padding-top: 2px;
      width: 1.2em;
    }

    .fake-selectlist-listbox {
      font-family: sans-serif;
      box-shadow: rgba(0, 0, 0, 0.13) 0px 12.8px 28.8px, rgba(0, 0, 0, 0.11) 0px 0px 9.2px;
      box-sizing: border-box;
      background-color: canvas;
      min-inline-size: anchor-size(self-inline);
      min-block-size: 1lh;
      position: fixed;
      width: fit-content;
      height: fit-content;
      color: canvastext;
      overflow: auto;
      border-width: initial;
      border-style: solid;
      border-color: initial;
      border-image: initial;
      border-radius: 0.25em;
      padding: 0.25em;
      margin: 0px;
      inset: auto;

      top: anchor(bottom);
    }

    .fake-selectlist option {
      font-size: 0.875em;
      padding: 0.25em;
    }
  `;
  return style;
}
