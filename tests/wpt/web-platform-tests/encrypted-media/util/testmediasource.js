function testmediasource(config) {

    return new Promise(function(resolve, reject) {
        // Fetch the media resources
        var fetches = [config.audioPath, config.videoPath].map(function(path) {
            return fetch(path).then(function(response) {
                if (!response.ok) throw new Error('Resource fetch failed');
                return response.arrayBuffer();
            });
        });

        Promise.all(fetches).then(function(resources) {
            config.audioMedia = resources[0];
            config.videoMedia = resources[1];

            // Create media source
            var source = new MediaSource();

            // Create and fill source buffers when the media source is opened
            source.addEventListener('sourceopen', onSourceOpen);

            function onSourceOpen(event) {
                var audioSourceBuffer = source.addSourceBuffer(config.audioType),
                    videoSourceBuffer = source.addSourceBuffer(config.videoType);

                audioSourceBuffer.appendBuffer(config.audioMedia);
                videoSourceBuffer.appendBuffer(config.videoMedia);

                function endOfStream() {
                    if (audioSourceBuffer.updating || videoSourceBuffer.updating) {
                        setTimeout(endOfStream, 250);
                    } else {
                        source.endOfStream();
                    }
                }

                endOfStream();
            }

            resolve(source);
        });
    });
}