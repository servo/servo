function resizeViewportTo(viewportSelector, width, height) {
	var iframe = document.querySelector(viewportSelector);
	// Commonly used trick to trigger a layout
	iframe.contentWindow.document.body.offsetTop;

	iframe.width = width;
	iframe.height = height;

	iframe.contentWindow.document.body.offsetTop;
}

function injectStylesInIFrame(styleSelector, frameSelector) {
	var style = document.querySelector(styleSelector),
		frame = document.querySelector(frameSelector);

	frame.contentWindow.addNewStyles(style.textContent);
}


if (window.parent != window) {
	// we're in an iframe, so expose the bits that allow setting styles inside
	window.addNewStyles = function (cssText) {
		var styleTag = document.createElement("style"),
			textNode = document.createTextNode(cssText);

		styleTag.appendChild(textNode);
		document.head.appendChild(styleTag);
	}
}
