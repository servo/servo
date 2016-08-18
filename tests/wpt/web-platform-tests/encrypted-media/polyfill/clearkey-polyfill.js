(function(){

    // Save platform functions that will be modified
    var _requestMediaKeySystemAccess = navigator.requestMediaKeySystemAccess.bind( navigator ),
        _setMediaKeys = HTMLMediaElement.prototype.setMediaKeys;

    // Allow us to modify the target of Events
    Object.defineProperties( Event.prototype, {
        target: {   get: function() { return this._target || this.currentTarget; },
                    set: function( newtarget ) { this._target = newtarget; } }
    } );

    var EventTarget = function(){
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

    function MediaKeySystemAccessProxy( keysystem, access, configuration )
    {
        this._keysystem = keysystem;
        this._access = access;
        this._configuration = configuration;
    }

    Object.defineProperties( MediaKeySystemAccessProxy.prototype, {
        keysystem: { get: function() { return this._keysystem; } }
    });

    MediaKeySystemAccessProxy.prototype.getConfiguration = function getConfiguration()
    {
        return this._configuration;
    };

    MediaKeySystemAccessProxy.prototype.createMediaKeys = function createMediaKeys()
    {
        return new Promise( function( resolve, reject ) {

            this._access.createMediaKeys()
            .then( function( mediaKeys ) { resolve( new MediaKeysProxy( mediaKeys ) ); })
            .catch( function( error ) { reject( error ); } );

        }.bind( this ) );
    };

    function MediaKeysProxy( mediaKeys )
    {
        this._mediaKeys = mediaKeys;
        this._sessions = [ ];
        this._videoelement = undefined;
        this._onTimeUpdateListener = MediaKeysProxy.prototype._onTimeUpdate.bind( this );
    }

    MediaKeysProxy.prototype._setVideoElement = function _setVideoElement( videoElement )
    {
        if ( videoElement !== this._videoelement )
        {
            if ( this._videoelement )
            {
                this._videoelement.removeEventListener( 'timeupdate', this._onTimeUpdateListener );
            }

            this._videoelement = videoElement;

            if ( this._videoelement )
            {
                this._videoelement.addEventListener( 'timeupdate', this._onTimeUpdateListener );
            }
        }
    };

    MediaKeysProxy.prototype._onTimeUpdate = function( event )
    {
        this._sessions.forEach( function( session  ) {

            if ( session._sessionType === 'persistent-usage-record' )
            {
                session._onTimeUpdate( event );
            }

        } );
    };

    MediaKeysProxy.prototype._removeSession = function _removeSession( session )
    {
        var index = this._sessions.indexOf( session );
        if ( index !== -1 ) this._sessions.splice( index, 1 );
    };

    MediaKeysProxy.prototype.createSession = function createSession( sessionType )
    {
        if ( !sessionType || sessionType === 'temporary' ) return this._mediaKeys.createSession();

        var session = new MediaKeySessionProxy( this, sessionType );
        this._sessions.push( session );

        return session;
    };

    MediaKeysProxy.prototype.setServerCertificate = function setServerCertificate( certificate )
    {
        return this._mediaKeys.setServerCertificate( certificate );
    };

    function MediaKeySessionProxy( mediaKeysProxy, sessionType )
    {
        EventTarget.call( this );

        this._mediaKeysProxy = mediaKeysProxy
        this._sessionType = sessionType;
        this._sessionId = "";

        // MediaKeySessionProxy states
        // 'created' - After initial creation
        // 'loading' - Persistent license session waiting for key message to load stored keys
        // 'active' - Normal active state - proxy all key messages
        // 'removing' - Release message generated, waiting for ack
        // 'closed' - Session closed
        this._state = 'created';

        this._closed = new Promise( function( resolve ) { this._resolveClosed = resolve; }.bind( this ) );
    }

    MediaKeySessionProxy.prototype = Object.create( EventTarget.prototype );

    Object.defineProperties( MediaKeySessionProxy.prototype, {

        sessionId:  { get: function() { return this._sessionId; } },
        expiration: { get: function() { return NaN; } },
        closed:     { get: function() { return this._closed; } },
        keyStatuses:{ get: function() { return this._session.keyStatuses; } },       // TODO this will fail if examined too early
        _kids:      { get: function() { return this._keys.map( function( key ) { return key.kid; } ); } },
    });

    MediaKeySessionProxy.prototype._createSession = function _createSession()
    {
        this._session = this._mediaKeysProxy._mediaKeys.createSession();

        this._session.addEventListener( 'message', MediaKeySessionProxy.prototype._onMessage.bind( this ) );
        this._session.addEventListener( 'keystatuseschange', MediaKeySessionProxy.prototype._onKeyStatusesChange.bind( this ) );
    };

    MediaKeySessionProxy.prototype._onMessage = function _onMessage( event )
    {
        switch( this._state )
        {
            case 'loading':
                this._session.update( toUtf8( { keys: this._keys } ) )
                .then( this._loaded );

                break;

            case 'active':
                this.dispatchEvent( event );
                break;

            default:
                // Swallow the event
                break;
        }
    };

    MediaKeySessionProxy.prototype._onKeyStatusesChange = function _onKeyStatusesChange( event )
    {
        switch( this._state )
        {
            case 'active' :
            case 'removing' :
                this.dispatchEvent( event );
                break;

            default:
                // Swallow the event
                break;
        }
    };

    MediaKeySessionProxy.prototype._onTimeUpdate = function _onTimeUpdate( event )
    {
        if ( !this._firstTime ) this._firstTime = Date.now();
        this._latestTime = Date.now();
        this._store();
    };

    MediaKeySessionProxy.prototype._queueMessage = function _queueMessage( messageType, message )
    {
        setTimeout( function() {

            var messageAsArray = toUtf8( message ).buffer;

            this.dispatchEvent( new MediaKeyMessageEvent( 'message', { messageType: messageType, message: messageAsArray } ) );

        }.bind( this ) );
    };

    function _storageKey( sessionId )
    {
        return sessionId;
    }

    MediaKeySessionProxy.prototype._store = function _store()
    {
        var data;

        if ( this._sessionType === 'persistent-usage-record' )
        {
            data = { kids: this._kids };
            if ( this._firstTime ) data.firstTime = this._firstTime;
            if ( this._latestTime ) data.latestTime = this._latestTime;
        }
        else
        {
            data = { keys: this._keys };
        }

        window.localStorage.setItem( _storageKey( this._sessionId ), JSON.stringify( data ) );
    };

    MediaKeySessionProxy.prototype._load = function _load( sessionId )
    {
        var store = window.localStorage.getItem( _storageKey( sessionId ) );
        if ( store === null ) return false;

        var data;
        try { data = JSON.parse( store ) } catch( error ) {
            return false;
        }

        if ( data.kids )
        {
            this._sessionType = 'persistent-usage-record';
            this._keys = data.kids.map( function( kid ) { return { kid: kid }; } );
            if ( data.firstTime ) this._firstTime = data.firstTime;
            if ( data.latestTime ) this._latestTime = data.latestTime;
        }
        else
        {
            this._sessionType = 'persistent-license';
            this._keys = data.keys;
        }

        return true;
    };

    MediaKeySessionProxy.prototype._clear = function _clear()
    {
        window.localStorage.removeItem( _storageKey( this._sessionId ) );
    };

    MediaKeySessionProxy.prototype.generateRequest = function generateRequest( initDataType, initData )
    {
        if ( this._state !== 'created' ) return Promise.reject( new InvalidStateError() );

        this._createSession();

        this._state = 'active';

        return this._session.generateRequest( initDataType, initData )
        .then( function() {
            this._sessionId = Math.random().toString(36).slice(2);
        }.bind( this ) );
    };

    MediaKeySessionProxy.prototype.load = function load( sessionId )
    {
        if ( this._state !== 'created' ) return Promise.reject( new InvalidStateError() );

        return new Promise( function( resolve, reject ) {

            try
            {
                if ( !this._load( sessionId ) )
                {
                    resolve( false );

                    return;
                }

                if ( this._sessionType === 'persistent-usage-record' )
                {
                    var msg = { kids: this._kids };
                    if ( this._firstTime ) msg.firstTime = this._firstTime;
                    if ( this._latestTime ) msg.latestTime = this._latestTime;

                    this._queueMessage( 'license-release', msg );

                    this._state = 'removing';

                    resolve( true );
                }
                else
                {
                    this._createSession();

                    this._state = 'loading';

                    var initData = { kids: this._kids };

                    this._session.generateRequest( 'keyids', toUtf8( initData ) );
                }
            }
            catch( error )
            {
                reject( error );
            }
        }.bind( this ) );
    };

    MediaKeySessionProxy.prototype.update = function update( response )
    {
        return new Promise( function( resolve, reject ) {

            switch( this._state )

                case 'active' :

                    var message = fromUtf8( response );

                    // JSON Web Key Set
                    this._keys = message.keys;

                    this._store();

                    resolve( this._session.update( response ) );

                    break;

                case 'removing' :

                    this._state = 'closed';

                    this._clear();

                    this._mediaKeysProxy._removeSession( this );

                    this._resolveClosed();

                    delete this._session;

                    resolve();

                    break;

                default:
                    reject( new InvalidStateError() );
            }

        }.bind( this ) );
    };

    MediaKeySessionProxy.prototype.close = function close()
    {
        if ( this._state === 'closed' ) return Promise.resolve();

        this._state = 'closed';

        this._mediaKeysProxy._removeSession( this );

        this._resolveClosed();

        var session = this._session;
        if ( !session ) return Promise.resolve();

        this._session = undefined;

        return session.close();
    };

    MediaKeySessionProxy.prototype.remove = function remove()
    {
        if ( this._state !== 'active' || !this._session ) return Promise.reject( new DOMException('InvalidStateError') );

        this._state = 'removing';

        this._mediaKeysProxy._removeSession( this );

        return this._session.close()
        .then( function() {

            var msg = { kids: this._kids };

            if ( this._sessionType === 'persistent-usage-record' )
            {
                if ( this._firstTime ) msg.firstTime = this._firstTime;
                if ( this._latestTime ) msg.latestTime = this._latestTime;
            }
            else
            {
                this._clear();
            }

            this._queueMessage( 'license-release', msg );

        }.bind( this ) )
    };

    HTMLMediaElement.prototype.setMediaKeys = function setMediaKeys( mediaKeys )
    {
        if ( mediaKeys instanceof MediaKeysProxy )
        {
            mediaKeys._setVideoElement( this );
            return _setMediaKeys.call( this, mediaKeys._mediaKeys );
        }
        else
        {
            return _setMediaKeys.call( this, mediaKeys );
        }
    };

    navigator.requestMediaKeySystemAccess = function( keysystem, configurations )
    {
        // First, see if this is supported by the platform
        return new Promise( function( resolve, reject ) {

            _requestMediaKeySystemAccess( keysystem, configurations )
            .then( function( access ) { resolve( access ); } )
            .catch( function( error ) {

                if ( error instanceof TypeError ) reject( error );

                if ( keysystem !== 'org.w3.clearkey' ) reject( error );

                if ( !configurations.some( is_persistent_configuration ) ) reject( error );

                // Shallow copy the configurations, swapping out the labels and omitting the sessiontypes
                var configurations_copy = configurations.map( function( config, index ) {

                    var config_copy = copy_configuration( config );
                    config_copy.label = index.toString();
                    return config_copy;

                } );

                // And try again with these configurations
                _requestMediaKeySystemAccess( keysystem, configurations_copy )
                .then( function( access ) {

                    // Create the supported configuration based on the original request
                    var configuration = access.getConfiguration(),
                        original_configuration = configurations[ configuration.label ];

                    // If the original configuration did not need persistent session types, then we're done
                    if ( !is_persistent_configuration( original_configuration ) ) resolve( access );

                    // Create the configuration that we will return
                    var returned_configuration = copy_configuration( configuration );

                    if ( original_configuration.label )
                        returned_configuration.label = original_configuration;
                    else
                        delete returned_configuration.label;

                    returned_configuration.sessionTypes = original_configuration.sessionTypes;

                    resolve( new MediaKeySystemAccessProxy( keysystem, access, returned_configuration ) );
                } )
                .catch( function( error ) { reject( error ); } );
            } );
        } );
    };

    function is_persistent_configuration( configuration )
    {
        return configuration.sessionTypes &&
                ( configuration.sessionTypes.indexOf( 'persistent-usage-record' ) !== -1
                || configuration.sessionTypes.indexOf( 'persistent-license' ) !== -1 );
    }

    function copy_configuration( src )
    {
        var dst = {};
        [ 'label', 'initDataTypes', 'audioCapabilities', 'videoCapabilities', 'distinctiveIdenfifier', 'persistentState' ]
        .forEach( function( item ) { if ( src[item] ) dst[item] = src[item]; } );
        return dst;
    }
}());