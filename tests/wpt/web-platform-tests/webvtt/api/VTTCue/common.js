function make_vtt_track(contents, test) {
    var track_blob = new Blob([contents], { type: 'text/vtt' });
    var track_url = URL.createObjectURL(track_blob);
    test.add_cleanup(function() {
        URL.revokeObjectURL(track_url);
    });
    return track_url;
}
