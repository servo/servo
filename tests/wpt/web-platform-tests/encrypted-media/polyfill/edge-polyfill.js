(function() {

    // This polyfill fixes the following problems with Edge browser
    // (1) To retrieve a persisted usage record, you must use session type 'persistent-release-message' instead of 'persistent-usage-record'
    // (2) To retrieve a persisted usage record, you must call remove() after calling load()
    // (3) On providing a license release acknowledgement, the session does not automatically close as is should
    // (4) Retrieval of the usage record at the end of an active session is not supported

    if ( navigator.userAgent.toLowerCase().indexOf('edge') > -1 ) {

        var _mediaKeySystemAccessCreateMediaKeys = MediaKeySystemAccess.prototype.createMediaKeys;
            _mediaKeysCreateSession = MediaKeys.prototype.createSession;

        // MediaKeySession proxy
        function MediaKeySession( mediaKeys, session )
        {
            EventTarget.call( this );

            this._mediaKeys = mediaKeys;
            this._session = session;
            this._sessionId = undefined;
            this._removing = false;

            session.addEventListener( 'message', this.dispatchEvent.bind( this ) );
            session.addEventListener( 'keystatuseschange', this.dispatchEvent.bind( this ) );
            session.closed.then( function() { if ( !this._removing ) this._resolveClosed(); }.bind ( this ) );

            this._closed = new Promise( function( resolve ) { this._resolveClosed = resolve; }.bind( this ) );
        }

        MediaKeySession.prototype = Object.create( EventTarget.prototype );

        Object.defineProperties( MediaKeySession.prototype, {
            sessionId:  { get: function() { return this._sessionId ? this._sessionId : this._session.sessionId; } },
            expiration: { get: function() { return this._session.expiration; } },
            closed:     { get: function() { return this._closed; } },
            keyStatuses:{ get: function() { return this._session.keyStatuses; } }
        });

        // load()
        //
        // Use a surrogate 'persistent-release-message' session to obtain the release message
        //
        MediaKeySession.prototype.load = function load( sessionId )
        {
            if ( this.sessionId ) return Promise.reject( new DOMException('InvalidAccessError') );

            this._surrogate = this._mediaKeys.createSession( 'persistent-release-message' );
            this._surrogate.addEventListener( 'message', this.dispatchEvent.bind( this ) );

            return this._surrogate.load( sessionId ).then( function( success ) {
                if (!success) return false;

                this._sessionId = sessionId;
                this._removing = true;
                this._session.close();

                return this._surrogate.remove().then( function() { return true; } );
            }.bind( this ) );
        };

        // remove()
        //
        // On an existing session, use a surrogate 'persistent-release-message' session to obtain the release message
        //
        MediaKeySession.prototype.remove = function remove()
        {
            if ( this._sessionId !== undefined ) return Promise.reject( new DOMException('InvalidAccessError') );
            if ( this.sessionId === undefined ) return Promise.reject( new DOMException('InvalidAccessError') );

            this._surrogate = this._mediaKeys.createSession( 'persistent-release-message' );
            this._surrogate.addEventListener( 'message', this.dispatchEvent.bind( this ) );
            this._removing = true;
            this._sessionId = this._session.sessionId;

            var self = this;

            return Promise.all( [ self._session.close(), self._session.closed ] ).then( function() {
                return self._surrogate.load( self._sessionId );
            }).then( function( success ) {
                if ( !success ) {
                    throw new DOMException('InvalidAccessError');
                }

                return self._surrogate.remove();
            }).then( function() { return true; } );
        }

        // update()
        //
        // For a normal session, pass through, otherwise update the surrogate and close the proxy
        MediaKeySession.prototype.update = function update( message )
        {
            if ( !this._removing ) return this._session.update( message );

            return this._surrogate.update( message ).then( function() {
                this._sessionId = undefined;
                this._resolveClosed();
            }.bind( this ) );
        };

        // close() - pass through
        //
        MediaKeySession.prototype.close = function close()
        {
            if ( !this._removing ) return this._session.close();
            this._resolveClosed();
            return Promise.resolve();
        };

        // generateRequest() - pass through
        //
        MediaKeySession.prototype.generateRequest = function generateRequest( initDataType, initData )
        {
            if ( this.sessionId ) Promise.reject( new DOMException('InvalidAccessError') );
            return this._session.generateRequest( initDataType, initData );
        };

        // Wrap PlayReady persistent-usage-record sessions in our Proxy
        MediaKeys.prototype.createSession = function createSession( sessionType ) {

            var session = _mediaKeysCreateSession.call( this, sessionType );
            if ( this._keySystem !== 'com.microsoft.playready' || sessionType !== 'persistent-usage-record' )
            {
                return session;
            }

            return new MediaKeySession( this, session );

        };

        //
        // Annotation polyfills - annotate not otherwise available data
        //

        // Annotate MediaKeys with the keysystem
        MediaKeySystemAccess.prototype.createMediaKeys = function createMediaKeys()
        {
            return _mediaKeySystemAccessCreateMediaKeys.call( this ).then( function( mediaKeys ) {
                mediaKeys._keySystem = this.keySystem;
                return mediaKeys;
            }.bind( this ) );
        };

        //
        // Utilities
        //

        // Allow us to modify the target of Events
        Object.defineProperties( Event.prototype, {
            target: {   get: function() { return this._target || this.currentTarget; },
                        set: function( newtarget ) { this._target = newtarget; } }
        } );

        // Make an EventTarget base class
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
