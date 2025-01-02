function styleExistsInSheet(styleText, sheet) {
  for (let rule of sheet.cssRules) {
    if (styleText == rule.cssText)
      return true;
    if (rule instanceof CSSImportRule) {
      if (rule.styleSheet && styleExistsInSheet(styleText, rule.styleSheet))
        return true;
    }
  }
  return false;
}

function styleExists(styleText) {
  for (let sheet of document.styleSheets) {
    if (styleExistsInSheet(styleText, sheet))
      return true;
  }
  return false;
}

