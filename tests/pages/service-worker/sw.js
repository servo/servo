self.addEventListener('activate', function(e) {
    console.log('Root service worker active');
});

self.addEventListener('fetch', function(e) {
    console.log("A fetch event detected by / service worker");
});

self.addEventListener('message', function(e) {
    console.log(e.data.num);
})
