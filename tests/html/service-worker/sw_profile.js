
self.addEventListener('activate', function(e) {
	console.log("profile service worker active");
});

self.addEventListener('fetch', function(e) {
	console.log("A fetch event detected by /profile service worker");
});