// Expect utf8decoder and utf8decoder to be TextEncoder('utf-8') and TextDecoder('utf-8') respectively

function MessageHandler( keysystem, content ) {
    this._keysystem = keysystem;
    this._content = content;
    this.messagehandler = MessageHandler.prototype.messagehandler.bind( this );
    this.servercertificate = undefined;
}

MessageHandler.prototype.messagehandler = function messagehandler( messageType, message )
{
    if ( messageType === 'license-request' )
    {
        var request = fromUtf8( message );

        var keys = request.kids.map( function( kid ) {

            var key;
            for( var i=0; i < this._content.keys.length; ++i )
            {
                if ( base64urlEncode( this._content.keys[ i ].kid ) === kid )
                {
                    key = base64urlEncode( this._content.keys[ i ].key );
                    break;
                }
            }

            return { kty: 'oct', kid: kid, k: key };

        }.bind( this ) );

        return Promise.resolve( toUtf8( { keys: keys } ) );
    }
    else if ( messageType === 'license-release' )
    {
        var release = fromUtf8( message );

        // TODO: Check the license release message here

        return Promise.resolve( toUtf8( { kids: release.kids } ) );
    }

    throw new TypeError( 'Unsupported message type for ClearKey' );
};

MessageHandler.prototype.createJWKSet = function createJWKSet(keyId, key) {
    var jwkSet = '{"keys":[';
    for (var i = 0; i < arguments.length; i++) {
        if (i != 0)
            jwkSet += ',';
        jwkSet += arguments[i];
    }
    jwkSet += ']}';
    return jwkSet;
};

MessageHandler.prototype.createJWK = function createJWK(keyId, key) {
    var jwk = '{"kty":"oct","alg":"A128KW","kid":"';
    jwk += base64urlEncode(keyId);
    jwk += '","k":"';
    jwk += base64urlEncode(key);
    jwk += '"}';
    return jwk;
};