(function(){
    if( navigator.userAgent.toLowerCase().indexOf('edge') === -1
            && navigator.userAgent.toLowerCase().indexOf('chrome') > -1){

        if ( ( /chrome\/([0-9]*)\./.exec( navigator.userAgent.toLowerCase() )[1] | 0 ) < 54 ) {

            // Work around https://bugs.chromium.org/p/chromium/issues/detail?id=622956
            // Chrome does not fire the empty keystatuschange event when a session is closed
            var _mediaKeySessionClose = MediaKeySession.prototype.close;
            var _mediaKeySessionKeyStatusesGetter = Object.getOwnPropertyDescriptor( MediaKeySession.prototype, 'keyStatuses' ).get;
            var _emptyMediaKeyStatusMap = { size: 0,
                                            has:    function() { return false; },
                                            get:    function() { return undefined; },
                                            entries:function() { return []; },          // this may not be correct, I think it should be some iterator thing
                                            keys:   function() { return []; },
                                            values: function() { return []; },
                                            forEach:function() { return; } };

            MediaKeySession.prototype.close = function close()
            {
                this.__closed = true;

                setTimeout( function() {
                        this.dispatchEvent( new Event( 'keystatuseschange' ) );
                }.bind( this ), 0 );

                return _mediaKeySessionClose.call( this );
            };

            Object.defineProperty( MediaKeySession.prototype, 'keyStatuses', { get: function() {

                return this.__closed ? _emptyMediaKeyStatusMap : _mediaKeySessionKeyStatusesGetter.call( this );

            } } );
        }
    }
}());
