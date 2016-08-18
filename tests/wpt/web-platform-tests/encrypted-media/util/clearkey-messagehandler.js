// Expect utf8decoder and utf8decoder to be TextEncoder('utf-8') and TextDecoder('utf-8') respectively
function messagehandler( keysystem, messageType, message )
{
    var contentmetadata = this;

    if ( messageType === 'license-request' )
    {
        var request = fromUtf8( message );

        var keys = request.kids.map( function( kid ) {

            var key;
            for( var i=0; i < contentmetadata.keys.length; ++i )
            {
                if ( base64urlEncode( contentmetadata.keys[ i ].kid ) === kid )
                {
                    key = base64urlEncode( contentmetadata.keys[ i ].key );
                    break;
                }
            }

            return { kty: 'oct', kid: kid, k: key };

        } );

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