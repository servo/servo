self.addEventListener('activate', function(e) {
	console.log("iframe service worker active");
});

self.addEventListener('fetch', function(e) {
	console.log("A fetch event detected by /iframe service worker");
});