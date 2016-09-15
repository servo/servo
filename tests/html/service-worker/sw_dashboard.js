self.addEventListener('activate', function(e) {
    console.log('Dashboard service worker active');
});

self.addEventListener('fetch', function(e) {
    console.log("A fetch event detected by /dashboard service worker");
});

self.addEventListener('message', function(e) {
    console.log(e.data.payload.msg + ' from '+ e.data.payload.worker_id);
})
