// Expect utf8decoder and utf8decoder to be TextEncoder('utf-8') and TextDecoder('utf-8') respectively
//
// drmconfig format:
// { <keysystem> : {    "serverURL"             : <the url for the server>,
//                      "httpRequestHeaders"    : <map of HTTP request headers>,
//                      "servertype"            : "microsoft" | "drmtoday",                 // affects how request parameters are formed
//                      "certificate"           : <base64 encoded server certificate> } }
//

drmconfig = {
    "com.widevine.alpha": [ {
        "serverURL": "https://lic.staging.drmtoday.com/license-proxy-widevine/cenc/",
        "servertype" : "drmtoday",
        "merchant" : "w3c-eme-test",
    } ],
    "com.microsoft.playready": [ {
        "serverURL": "http://playready-testserver.azurewebsites.net/rightsmanager.asmx",
        "servertype": "microsoft",
        "sessionTypes" : [ "persistent-usage-record" ],
        "certificate" : "Q0hBSQAAAAEAAAUEAAAAAAAAAAJDRVJUAAAAAQAAAfQAAAFkAAEAAQAAAFjt9G6KdSncCkrjbTQPN+/2AAAAAAAAAAAAAAAJIPbrW9dj0qydQFIomYFHOwbhGZVGP2ZsPwcvjh+NFkP/////AAAAAAAAAAAAAAAAAAAAAAABAAoAAABYxw6TjIuUUmvdCcl00t4RBAAAADpodHRwOi8vcGxheXJlYWR5LmRpcmVjdHRhcHMubmV0L3ByL3N2Yy9yaWdodHNtYW5hZ2VyLmFzbXgAAAAAAQAFAAAADAAAAAAAAQAGAAAAXAAAAAEAAQIAAAAAADBRmRRpqV4cfRLcWz9WoXIGZ5qzD9xxJe0CSI2mXJQdPHEFZltrTkZtdmurwVaEI2etJY0OesCeOCzCqmEtTkcAAAABAAAAAgAAAAcAAAA8AAAAAAAAAAVEVEFQAAAAAAAAABVNZXRlcmluZyBDZXJ0aWZpY2F0ZQAAAAAAAAABAAAAAAABAAgAAACQAAEAQGHic/IPbmLCKXxc/MH20X/RtjhXH4jfowBWsQE1QWgUUBPFId7HH65YuQJ5fxbQJCT6Hw0iHqKzaTkefrhIpOoAAAIAW+uRUsdaChtq/AMUI4qPlK2Bi4bwOyjJcSQWz16LAFfwibn5yHVDEgNA4cQ9lt3kS4drx7pCC+FR/YLlHBAV7ENFUlQAAAABAAAC/AAAAmwAAQABAAAAWMk5Z0ovo2X0b2C9K5PbFX8AAAAAAAAAAAAAAARTYd1EkpFovPAZUjOj2doDLnHiRSfYc89Fs7gosBfar/////8AAAAAAAAAAAAAAAAAAAAAAAEABQAAAAwAAAAAAAEABgAAAGAAAAABAAECAAAAAABb65FSx1oKG2r8AxQjio+UrYGLhvA7KMlxJBbPXosAV/CJufnIdUMSA0DhxD2W3eRLh2vHukIL4VH9guUcEBXsAAAAAgAAAAEAAAAMAAAABwAAAZgAAAAAAAAAgE1pY3Jvc29mdAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgFBsYXlSZWFkeSBTTDAgTWV0ZXJpbmcgUm9vdCBDQQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgDEuMC4wLjEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEACAAAAJAAAQBArAKJsEIDWNG5ulOgLvSUb8I2zZ0c5lZGYvpIO56Z0UNk/uC4Mq3jwXQUUN6m/48V5J/vuLDhWu740aRQc1dDDAAAAgCGTWHP8iVuQixWizwoABz7PhUnZYWEugUht5sYKNk23h2Cao/D5uf6epDVyilG8fZKLvufXc/+fkNOtEKT+sWr"
    },
    {
        "serverURL": "http://playready.directtaps.net/pr/svc/rightsmanager.asmx",
        "servertype": "microsoft",
        "sessionTypes" : [ "persistent-usage-record" ],
        "certificate" : "Q0hBSQAAAAEAAAUEAAAAAAAAAAJDRVJUAAAAAQAAAfQAAAFkAAEAAQAAAFjt9G6KdSncCkrjbTQPN+/2AAAAAAAAAAAAAAAJIPbrW9dj0qydQFIomYFHOwbhGZVGP2ZsPwcvjh+NFkP/////AAAAAAAAAAAAAAAAAAAAAAABAAoAAABYxw6TjIuUUmvdCcl00t4RBAAAADpodHRwOi8vcGxheXJlYWR5LmRpcmVjdHRhcHMubmV0L3ByL3N2Yy9yaWdodHNtYW5hZ2VyLmFzbXgAAAAAAQAFAAAADAAAAAAAAQAGAAAAXAAAAAEAAQIAAAAAADBRmRRpqV4cfRLcWz9WoXIGZ5qzD9xxJe0CSI2mXJQdPHEFZltrTkZtdmurwVaEI2etJY0OesCeOCzCqmEtTkcAAAABAAAAAgAAAAcAAAA8AAAAAAAAAAVEVEFQAAAAAAAAABVNZXRlcmluZyBDZXJ0aWZpY2F0ZQAAAAAAAAABAAAAAAABAAgAAACQAAEAQGHic/IPbmLCKXxc/MH20X/RtjhXH4jfowBWsQE1QWgUUBPFId7HH65YuQJ5fxbQJCT6Hw0iHqKzaTkefrhIpOoAAAIAW+uRUsdaChtq/AMUI4qPlK2Bi4bwOyjJcSQWz16LAFfwibn5yHVDEgNA4cQ9lt3kS4drx7pCC+FR/YLlHBAV7ENFUlQAAAABAAAC/AAAAmwAAQABAAAAWMk5Z0ovo2X0b2C9K5PbFX8AAAAAAAAAAAAAAARTYd1EkpFovPAZUjOj2doDLnHiRSfYc89Fs7gosBfar/////8AAAAAAAAAAAAAAAAAAAAAAAEABQAAAAwAAAAAAAEABgAAAGAAAAABAAECAAAAAABb65FSx1oKG2r8AxQjio+UrYGLhvA7KMlxJBbPXosAV/CJufnIdUMSA0DhxD2W3eRLh2vHukIL4VH9guUcEBXsAAAAAgAAAAEAAAAMAAAABwAAAZgAAAAAAAAAgE1pY3Jvc29mdAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgFBsYXlSZWFkeSBTTDAgTWV0ZXJpbmcgUm9vdCBDQQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgDEuMC4wLjEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEACAAAAJAAAQBArAKJsEIDWNG5ulOgLvSUb8I2zZ0c5lZGYvpIO56Z0UNk/uC4Mq3jwXQUUN6m/48V5J/vuLDhWu740aRQc1dDDAAAAgCGTWHP8iVuQixWizwoABz7PhUnZYWEugUht5sYKNk23h2Cao/D5uf6epDVyilG8fZKLvufXc/+fkNOtEKT+sWr"
    },
    {
        "serverURL": "https://lic.staging.drmtoday.com/license-proxy-headerauth/drmtoday/RightsManager.asmx",
        "servertype" : "drmtoday",
        "sessionTypes" : [ "temporary", "persistent-usage-record", "persistent-license" ],
        "merchant" : "w3c-eme-test"
    } ]
};

function MessageHandler( keysystem, content, sessionType ) {
    sessionType = sessionType || "temporary";

    this._keysystem = keysystem;
    this._content = content;
    this._sessionType = sessionType;
    this._drmconfig = drmconfig[ this._keysystem ].filter( function( drmconfig ) {
        return drmconfig.sessionTypes === undefined || ( drmconfig.sessionTypes.indexOf( sessionType ) !== -1 );
    } )[ 0 ];

    this.messagehandler = MessageHandler.prototype.messagehandler.bind( this );
    if ( this._drmconfig && this._drmconfig.certificate ) {
        this.servercertificate = stringToUint8Array( atob( this._drmconfig.certificate ) );
    }
}

MessageHandler.prototype.messagehandler = function messagehandler( messageType, message) {

    // For the DRM Today server, mapping between Key System messages and server protocol messages depends on
    // the Key System, so we provide key-system-specific functions here to perform the mapping.
    //
    // For the Microsoft server, the mapping for PlayReady is the same as for the DRM Today server
    //
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
                return String.fromCharCode.apply(null, new Uint16Array(response));
            },
            getLicenseRequestFromMessage: function(message) {
                var msg,
                    xmlDoc;
                var licenseRequest = null;
                var parser = new DOMParser();
                var dataview = new Uint16Array(message);

                msg = String.fromCharCode.apply(null, dataview);

                xmlDoc = parser.parseFromString(msg, 'application/xml');

                if (xmlDoc.getElementsByTagName('Challenge')[0]) {
                    var Challenge = xmlDoc.getElementsByTagName('Challenge')[0].childNodes[0].nodeValue;
                    if (Challenge) {
                        licenseRequest = atob(Challenge);
                    }
                }
                return licenseRequest;
            },
            getRequestHeadersFromMessage: function(message) {
                var msg,
                    xmlDoc;
                var headers = {};
                var parser = new DOMParser();
                var dataview = new Uint16Array(message);

                msg = String.fromCharCode.apply(null, dataview);
                xmlDoc = parser.parseFromString(msg, 'application/xml');

                var headerNameList = xmlDoc.getElementsByTagName('name');
                var headerValueList = xmlDoc.getElementsByTagName('value');
                for (var i = 0; i < headerNameList.length; i++) {
                    headers[headerNameList[i].childNodes[0].nodeValue] = headerValueList[i].childNodes[0].nodeValue;
                }
                // some versions of the PlayReady CDM return 'Content' instead of 'Content-Type',
                // but the license server expects 'Content-Type', so we fix it up here.
                if (headers.hasOwnProperty('Content')) {
                    headers['Content-Type'] = headers.Content;
                    delete headers.Content;
                }
                return headers;
            }
        }
    };

    // License request parameters are communicated to the DRM Today and Microsoft servers in different ways,
    // using a custom HTTP headers (DRM Today) and URL parameters (Microsoft).
    const serverTypes = {
        'drmtoday': {
            constructLicenseRequestUrl : function( serverURL, sessionType, messageType, content ) {
                return serverURL;
            },
            getCustomHeaders : function( drmconfig, sessionType, messageType ) {
                var customHeader = {    merchant: drmconfig.merchant,
                                        userId: "purchase",
                                        sessionId: "a0d0f0p" + ( ( sessionType === 'persistent-license' ) ? "1" : "0" ) };
                return { "dt-custom-data" : btoa( JSON.stringify( customHeader ) ) };
            }
        },
        'microsoft': {
            constructLicenseRequestUrl : function( serverURL, sessionType, messageType, content ) {
                if ( messageType !== 'license-request' ) {
                    return serverURL;
                }

                var url = serverURL + "?";
                if ( sessionType === 'temporary' || sessionType === 'persistent-usage-record' ) {
                    url += "UseSimpleNonPersistentLicense=1&";
                }
                if ( sessionType === 'persistent-usage-record' ) {
                    url += "SecureStop=1&";
                }
                url += "PlayEnablers=B621D91F-EDCC-4035-8D4B-DC71760D43E9&";    // disable output protection
                url += "ContentKey=" + btoa(String.fromCharCode.apply(null, content.keys[0].key));
                return url;
            },
            getCustomHeaders : function() { return {}; }
        }
    };

    return new Promise(function(resolve, reject) {
        var keysystemfns = keySystems[this._keysystem],
            serverfns,
            url = undefined,
            requestheaders = {},
            credentials = undefined;

        if ( !this._drmconfig || !keysystemfns || !this._drmconfig.servertype || !serverTypes[this._drmconfig.servertype] ) {
            reject('Unsupported Key System');
            return;
        }

        serverfns = serverTypes[this._drmconfig.servertype];

        if ( !this._drmconfig.serverURL ) {
            reject('Undefined serverURL');
            return;
        }

        url = serverfns.constructLicenseRequestUrl( this._drmconfig.serverURL, this._sessionType, messageType, this._content );

        // Ensure valid license server URL
        if (!url) {
            reject('No license server URL specified!');
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
                    requestheaders[key] = headers[key];
                }
            }
        };

        updateHeaders(serverfns.getCustomHeaders( this._drmconfig, this._sessionType, messageType ) );
        updateHeaders(keysystemfns.getRequestHeadersFromMessage(message));

        // Set withCredentials property from server
        if ( this._drmconfig.withCredentials ) {
            credentials = 'include';
        }

        fetch(url, {
            method: 'POST',
            headers: requestheaders,
            credentials: credentials,
            body: keysystemfns.getLicenseRequestFromMessage(message)
        }).then(function(fetchresponse) {
            if(fetchresponse.status !== 200) {
                reject( this._keysystem + ' update, XHR status is "' + fetchresponse.statusText
                            + '" (' + fetchresponse.status + '), expected to be 200. readyState is ' + fetchresponse.readyState + '.'
                            + ' Response is ' + ((fetchresponse) ? keysystemfns.getErrorResponse(fetchresponse) : 'NONE' ));
                return;
            }

            if(keysystemfns.responseType === 'json') {
                return fetchresponse.json();
            } else if(keysystemfns.responseType === 'arraybuffer') {
                return fetchresponse.arrayBuffer();
            }
        }.bind( this )).then(function(response){
            resolve(keysystemfns.getLicenseMessage(response));
        }).catch(reject);
    }.bind( this ));
};
