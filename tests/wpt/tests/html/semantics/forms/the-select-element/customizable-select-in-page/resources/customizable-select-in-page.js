function updateCustomizableSelectInPageSize() {
  document.querySelectorAll('.customizable-select-in-page').forEach(select => {
    const size = Number(select.getAttribute('size'));
    if (size) {
      let optionHeight = 0;
      select.querySelectorAll('.customizable-select-option').forEach(option => {
        const rect = option.getBoundingClientRect();
        optionHeight = Math.max(optionHeight, rect.height);
      });
      select.style.height = (optionHeight * size) + 'px';
    }
  });
}
