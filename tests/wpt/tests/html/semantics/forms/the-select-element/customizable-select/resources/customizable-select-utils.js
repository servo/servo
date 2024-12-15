function populateFallbackButtonIcon() {
  document.querySelectorAll('.customizable-select-button-icon').forEach(element => {
    element.innerHTML =
      `<svg viewBox="0 0 20 16" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M4 6 L10 12 L 16 6"></path>
      </svg>`;
  });
}
