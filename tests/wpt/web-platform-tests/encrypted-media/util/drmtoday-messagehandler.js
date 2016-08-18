// Expect utf8decoder and utf8decoder to be TextEncoder('utf-8') and TextDecoder('utf-8') respectively
function messagehandler(keysystem, messageType, message) {

    var contentmetadata = this;

    const keySystems = {
        'com.widevine.alpha': {
            responseType: 'json',
            getLicenseMessage: function(response) {
                return base64DecodeToUnit8Array( response.license );
            },
            getErrorResponse: function(response) {
                return response;
            },
            getLicenseRequestFromMessage: function(message) {
                return new Uint8Array(message);
            },
            getRequestHeadersFromMessage: function(/*message*/) {
                return null;
            }
        },
        'com.microsoft.playready': {
            responseType: 'arraybuffer',
            getLicenseMessage: function(response) {
                return response;
            },
            getErrorResponse: function(response) {
                return String.fromCharCode.apply(null, new Uint8Array(response));
            },
            getLicenseRequestFromMessage: function(message) {
                // TODO: Add playready specific stuff.
                return message;
            },
            getRequestHeadersFromMessage: function(message) {
                // TODO: Add playready specific stuff.
                return null;
            }
        }
    };

    return new Promise(function(resolve, reject) {

        readDrmConfig().then(function(response) {

            var protData = response[keysystem],
                url = undefined,
                reqheaders = {},
                credentials = undefined;

            if (protData) {
                if (protData.serverURL) {
                    url = protData.serverURL;
                } else {
                    reject('Undefined serverURL');
                    return;
                }
            } else {
                reject('Unsupported keySystem');
                return;
            }

            // Ensure valid license server URL
            if (!url) {
                reject('DRM: No license server URL specified!');
                return;
            }

            // Set optional XMLHttpRequest headers from protection data and message
            var updateHeaders = function(headers) {
                var key;
                if (headers) {
                    for (key in headers) {
                        if ('authorization' === key.toLowerCase()) {
                            credentials = 'include';
                        }
                        reqheaders[key] = headers[key];
                    }
                }
            };

            if (protData) {
                updateHeaders(protData.httpRequestHeaders);
            }

            updateHeaders(keySystems[keysystem].getRequestHeadersFromMessage(message));

            // Set withCredentials property from protData
            if (protData && protData.withCredentials) {
                credentials = 'include';
            }

            fetch(url, {
                method: 'POST',
                headers: reqheaders,
                credentials: credentials,
                body: keySystems[keysystem].getLicenseRequestFromMessage(message)
            }).then(function(response) {
                if(response.status !== 200) {
                    reject('DRM: ' + keysystem + ' update, XHR status is "' + response.statusText + '" (' + response.status + '), expected to be 200. readyState is ' + response.readyState + '.  Response is ' + ((response) ? keySystems[keysystem].getErrorResponse(response) : 'NONE'));
                    return;
                } else {
                    return response.json();
                }
            }).then(function(response){
                resolve(keySystems[keysystem].getLicenseMessage(response));
            }).catch(function(error) {
                reject(error);
                return;
            });
        });
    });
}

function readDrmConfig() {
    return fetch("/encrypted-media/content/drmconfig.json").then(function(response) {
        return response.json();
    });
}