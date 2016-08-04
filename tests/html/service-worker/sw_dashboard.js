importScripts('dashboard_helper.js');

self.addEventListener('activate', function(e) {
	status_from_dashboard();
});

self.addEventListener('fetch', function(e) {
	console.log("A fetch event detected by /dashboard service worker");
});