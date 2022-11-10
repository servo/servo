// This script is intended to be imported into a worker's script, and provides
// common preparation for multiple test cases. Errors encountered are either
// postMessaged with subject of messageSubject.ERROR, or in the case of failed
// mediaLoadPromise, result in promise rejection.

importScripts("mediasource-message-util.js");

if (!this.MediaSource)
  postMessage({ subject: messageSubject.ERROR, info: "MediaSource API missing from Worker" });

let MEDIA_LIST = [
  {
    url: '../mp4/test.mp4',
    type: 'video/mp4; codecs="mp4a.40.2,avc1.4d400d"',
  },
  {
    url: '../webm/test.webm',
    type: 'video/webm; codecs="vp8, vorbis"',
  },
];

class MediaSourceWorkerUtil {
  constructor() {
    this.mediaSource = new MediaSource();

    // Find supported test media, if any.
    this.foundSupportedMedia = false;
    for (let i = 0; i < MEDIA_LIST.length; ++i) {
      this.mediaMetadata = MEDIA_LIST[i];
      if (MediaSource.isTypeSupported(this.mediaMetadata.type)) {
        this.foundSupportedMedia = true;
        break;
      }
    }

    // Begin asynchronous fetch of the test media.
    if (this.foundSupportedMedia) {
      this.mediaLoadPromise = MediaSourceWorkerUtil.loadBinaryAsync(this.mediaMetadata.url);
    } else {
      postMessage({ subject: messageSubject.ERROR, info: "No supported test media" });
    }
  }

  static loadBinaryAsync(url) {
    return new Promise((resolve, reject) => {
      let request = new XMLHttpRequest();
      request.open("GET", url, true);
      request.responseType = "arraybuffer";
      request.onerror = event => { reject(event); };
      request.onload = () => {
        if (request.status != 200) {
          reject("Unexpected loadData_ status code : " + request.status);
        }
        let response = new Uint8Array(request.response);
        resolve(response);
      };
      request.send();
    });
  }
}
