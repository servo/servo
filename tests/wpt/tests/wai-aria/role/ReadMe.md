
# WPT Roles Tests

/wai-aria/roles/ includes various files broken up by test type

- **roles.html** covers simple assignment/verification of most core WAI-ARIA roles, and includes a list of cross-references to other specific files and spec directories.
- role testing of *host language* implicit roles (E.g., `<main> -> main`) are in other directories (E.g., [html-aam](https://github.com/web-platform-tests/interop-accessibility/issues/13))
- role testing of **ARIA extension specs** are in other directories (E.g., [graphics-aria](https://github.com/web-platform-tests/interop-accessibility/issues/9))
- basic.html was the first to ensure basic test coverage of webdriver getcomputedrole
- other context-dependent role tests, error handling, and edge cases are covered in separate files
  - list-roles.html
  - region-roles.html
  - grid, listbox, menu, tree, etc
  - fallback roles
  - invalid roles
  - error handling, etc.
