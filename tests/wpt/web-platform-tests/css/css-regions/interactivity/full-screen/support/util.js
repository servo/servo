function makeFullScreen(selector) {
	var element = document.querySelector(selector);
	if (selector) {
		selector.requestFullscreen();
	}
}

function makeFullScreenToggle(selector, targetSelector) {
	var button = document.querySelector(selector);
	button.addEventListener("click", function() {
		var element = document.querySelector(targetSelector);
		if (element.requestFullscreen) {
			element.requestFullscreen();
		} else {
			document.querySelector("#fail-marker").style.visibility = "visible";
		}
	})
}