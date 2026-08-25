(function() {

    // This polyfill fixes the following problems with Edge browser
    // (1) Various maplike methods for keystatuses are not supported or suported incorrectly
    // (2) Key Ids exposed in keystatuses are incorrect (byte swaps)
    if ( navigator.userAgent.toLowerCase().indexOf('edge') > -1 ) {
        ///////////////////////////////////////////////////////////////////////////////////////////////
        // The following function is the core of this JS patch. The rest of this file is infrastructure
        // required to enable this function
        ///////////////////////////////////////////////////////////////////////////////////////////////
        function _proxyKeyStatusesChange( event ) {
            this._keyStatuses.clear();
            var keyStatuses = [];
            this._session.keyStatuses.forEach( function( keyId, status ) {
                var newKeyId = new Uint8Array( keyId );

                function swap( arr, a, b ) { var t = arr[a]; arr[a] = arr[b]; arr[b] = t; }
                swap( newKeyId, 0, 3 );
                swap( newKeyId, 1, 2 );
                swap( newKeyId, 4, 5 );
                swap( newKeyId, 6, 7 );

                keyStatuses.push( { key: newKeyId, status: status, ord: arrayBufferAsString( newKeyId ) } );
            });

            function lexicographical( a, b ) { return a < b ? -1 : a === b ? 0 : +1; }
            function lexicographicalkey( a, b ) { return lexicographical( a.ord, b.ord ); }

            keyStatuses.sort( lexicographicalkey ).forEach( function( obj ) {
                this._keyStatuses._set( obj.key, obj.status );
            }.bind( this ) );

            this.dispatchEvent( event );
        };
        ///////////////////////////////////////////////////////////////////////////////////////////////

        // Override MediaKeys.createSession
        var _mediaKeysCreateSession = MediaKeys.prototype.createSession;
        MediaKeys.prototype.createSession = function ( sessionType ) {
            return new MediaKeySession( _mediaKeysCreateSession.call( this, sessionType ) );
        };

        // MediaKeySession proxy
        function MediaKeySession( session ) {
            EventTarget.call( this );
            this._session = session;
            this._keyStatuses = new MediaKeyStatusMap();
            this._session.addEventListener("keystatuseschange",this._onKeyStatusesChange.bind(this));
            this._session.addEventListener("message",this.dispatchEvent.bind(this));
        }

        MediaKeySession.prototype = Object.create( EventTarget.prototype );

        Object.defineProperties( MediaKeySession.prototype, {
            sessionId:  { get: function() { return this._session.sessionId; } },
            expiration: { get: function() { return this._session.expiration; } },
            closed:     { get: function() { return this._session.closed; } },
            keyStatuses:{ get: function() { return this._keyStatuses; } }
        });

        [ "generateRequest", "load", "update", "remove", "close" ].forEach( function( fnname ) {
            MediaKeySession.prototype[ fnname ] = function() {
                return window.MediaKeySession.prototype[ fnname ].apply( this._session, arguments );
            }
        } );

        MediaKeySession.prototype._onKeyStatusesChange = _proxyKeyStatusesChange;

        // MediaKeyStatusMap proxy
        //
        // We need a proxy class to replace the broken MediaKeyStatusMap one. We cannot use a
        // regular Map directly because we need get and has methods to compare by value not
        // as references.
        function MediaKeyStatusMap() { this._map = new Map(); }

        Object.defineProperties( MediaKeyStatusMap.prototype, {
            size:               { get: function() { return this._map.size; } },
            forEach:            { get: function() { return function( f ) { return this._map.forEach( f ); } } },
            entries:            { get: function() { return function() { return this._map.entries(); } } },
            values:             { get: function() { return function() { return this._map.values(); } } },
            keys:               { get: function() { return function() { return this._map.keys(); } } },
            clear:              { get: function() { return function() { return this._map.clear(); } } } } );

        MediaKeyStatusMap.prototype[ Symbol.iterator ] = function() { return this._map[ Symbol.iterator ]() };

        MediaKeyStatusMap.prototype.has = function has( keyId ) {
            for ( var k of this._map.keys() ) { if ( arrayBufferEqual( k, keyId ) ) return true; }
            return false;
        };

        MediaKeyStatusMap.prototype.get = function get( keyId ) {
            for ( var k of this._map.entries() ) { if ( arrayBufferEqual( k[ 0 ], keyId ) ) return k[ 1 ]; }
        };

        MediaKeyStatusMap.prototype._set = function _set( keyId, status ) {
            this._map.set( new Uint8Array( keyId ), status );
        };

        function arrayBufferEqual(buf1, buf2)
        {
            if (buf1.byteLength !== buf2.byteLength) return false;
            var a1 = Array.from( new Int8Array(buf1) ), a2 = Array.from( new Int8Array(buf2) );
            return a1.every( function( x, i ) { return x === a2[i]; } );
        }

        // EventTarget
        function EventTarget(){
            this.listeners = {};
        };

        EventTarget.prototype.listeners = null;

        EventTarget.prototype.addEventListener = function(type, callback){
            if(!(type in this.listeners)) {
                this.listeners[type] = [];
            }
            this.listeners[type].push(callback);
        };

        EventTarget.prototype.removeEventListener = function(type, callback){
            if(!(type in this.listeners)) {
                return;
            }
            var stack = this.listeners[type];
            for(var i = 0, l = stack.length; i < l; i++){
                if(stack[i] === callback){
                    stack.splice(i, 1);
                    return this.removeEventListener(type, callback);
                }
            }
        };

        EventTarget.prototype.dispatchEvent = function(event){
            if(!(event.type in this.listeners)) {
                return;
            }
            var stack = this.listeners[event.type];
            event.target = this;
            for(var i = 0, l = stack.length; i < l; i++) {
                stack[i].call(this, event);
            }
        };
    }
})();
