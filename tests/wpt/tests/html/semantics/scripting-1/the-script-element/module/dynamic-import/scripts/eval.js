// import()s in eval are resolved relative to the script.
eval(`import('../../imports-a.js?label=' + window.label).then(window.continueTest, window.errorTest)`);
