function replaceChildElement(newChild, oldChild) {
  oldChild.parentElement.replaceChild(newChild, oldChild);
}

function createFakeSelectlist(selectedValueText) {
  const selectlist = document.createElement('div');
  selectlist.classList.add('fake-selectlist-internal-selectlist-button');
  selectlist.innerHTML = `
    <div class="fake-selectlist-selected-value"></div>
    <div class="fake-selectlist-internal-selectlist-button-icon"></div>
    <style>
    .fake-selectlist-internal-selectlist-button {
      align-items: center;
      appearance: none;
      background-color: Field;
      border: 1px solid ButtonBorder;
      border-radius: 0.25em;
      box-sizing: border-box;
      box-shadow: none;
      color: ButtonText;
      cursor: default;
      display: inline-flex;
      font: -webkit-small-control;
      font-size: .875em;
      overflow-x:hidden;
      overflow-y:hidden;
      padding: 0.25em;
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
    </style>
  `;

  if (selectedValueText) {
    selectlist.querySelector('.fake-selectlist-selected-value').textContent = selectedValueText;
  }

  return selectlist;
}
