(function() {

    if ( /CrKey\/[0-9]+\.[0-9a-z]+\.[0-9a-z]+/i.exec( navigator.userAgent ) ) {

        var castscript = document.createElement('script');
        castscript.type = 'text/javascript';
        castscript.src = 'https://www.gstatic.com/cast/sdk/libs/receiver/2.0.0/cast_receiver.js'
        document.head.appendChild( castscript );

        var _requestMediaKeySystemAccess = navigator.requestMediaKeySystemAccess.bind( navigator ),
            _setMediaKeys = HTMLMediaElement.prototype.setMediaKeys,
            _load = MediaKeySession.prototype.load;

        MediaKeySession.prototype.load = function load()
        {
            return _load.call( this ).then( function( success )
            {
                return success ? this.remove() : false;
            }.bind( this ) );
        };

        function MediaKeys( mediaKeys )
        {
            this._mediaKeys = mediaKeys;
        }

        MediaKeys.prototype.setServerCertificate = function setServerCertificate( certificate )
        {
            return this._mediaKeys.setServerCertificate( certificate );
        };

        MediaKeys.prototype.createSession = function createSession( sessionType ) {

            if ( sessionType === 'persistent-usage-record' )
            {
                return cast.receiver.eme.KeySession.createSession( this._mediaKeys, 'persistent-release-message' );
            }

            return this._mediaKeys.createSession( sessionType );
        };

        function MediaKeySystemAccess( access )
        {
            this._access = mediaKeySystemAccess;
        }

        Object.defineProperty( MediaKeySystemAccess.prototype, 'keySystem', { get: function() { return this._access.keySystem; } } );

        MediaKeySystemAccess.prototype.getConfiguration = function getConfiguration() { return this._access.getConfiguration(); };

        MediaKeySystemAccess.prototype.createMediaKeys = function createMediaKeys() {

            return this._access.createMediaKey().then( function( mediaKeys ) { return new MediaKeys( mediaKeys ); } );

        };

        HTMLMediaElement.prototype.setMediaKeys = function setMediaKeys( mediaKeys )
        {
            if ( mediaKeys instanceof MediaKeys )
            {
                return _setMediaKeys.call( this, mediaKeys._mediaKeys );
            }
            else
            {
                return _setMediaKeys.call( this, mediaKeys );
            }
        };

        navigator.requestMediaKeySystemAccess = function requestMediaKeySystemAccess( keysystem, supportedConfigurations ) {

            if ( keysystem !== 'com.chromecast.playready' )
            {
                return _requestMediaKeySystemAccess( keysystem, supportedConfigurations );
            }

            return _requestMediaKeySystemAccess( keysystem, supportedConfigurations )
            .then( function( access ) { return new MediaKeySystemAccess( access ); } );
        };
    }
})();