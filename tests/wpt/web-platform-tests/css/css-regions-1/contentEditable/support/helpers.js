document.addEventListener("DOMContentLoaded", function() {
	// Mare sure we're styling with CSS
	document.execCommand("styleWithCss", true, null);
});

function insertText(selector, newText) {
	var selection = window.getSelection(),
		insertionPoint = document.querySelector(selector),
		range = document.createRange();

	range.selectNode(insertionPoint);

	selection.removeAllRanges();
	selection.addRange(range);

	document.execCommand("insertText", true, newText);
}

function duplicate(selector) {
	var selection = window.getSelection(),
		insertionPoint = document.querySelector(selector),
		range = document.createRange();

	range.selectNode(insertionPoint);
	selection.removeAllRanges();
	selection.addRange(range);

	document.execCommand("copy", true, null);

	selection.removeAllRanges();
	document.execCommand("paste", true, null);
}