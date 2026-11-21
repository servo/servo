
self.addEventListener('activate', function(e) {
    console.log("profile service worker active");
});

self.addEventListener('fetch', function(e) {
    console.log("A fetch event detected by /profile service worker");
});

self.addEventListener('message', function(e) {
    console.log(e.data.payload.msg + ' from '+ e.data.payload.worker_id);
})
