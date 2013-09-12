var longcats = window.document.getElementsByTagName("img");
var longcat_top = longcats[0];
var longcat_mid = longcats[1];
var longcat_bot = longcats[2];

function wait_for_img_load(f) {
	if (longcat_top.width != 0 && longcat_mid.width != 0 && longcat_bot.width != 0) {
		f();
	} else {
		window.setTimeout(function() { wait_for_img_load(f) }, 1);
	}
}

wait_for_img_load(function() {
	var count = 0;
	function elongate() {
		var height = Math.round((Math.cos(count + Math.PI) + 1) * 100 + 20);
		count += 0.2;
		longcat_mid.height = height;
		longcat_mid.width = 600;
		window.setTimeout(function() { elongate() }, 50);
	}
	elongate();
});
