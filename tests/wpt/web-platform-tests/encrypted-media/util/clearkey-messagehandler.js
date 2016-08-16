// Expect utf8decoder and utf8decoder to be TextEncoder('utf-8') and TextDecoder('utf-8') respectively
function messagehandler( keysystem, messageType, message )
{
    var keys = { 'AAAAAAPS_EEAAAAAAAAAAA' : 'rzQTSR-sLD46a4jgU4RCBg' };

    var self = this;

    if ( messageType === 'license-request' )
    {
        var request = fromUtf8( message );

        var keys = request.kids.map( function( kid ) {

            return { kty: 'oct', kid: kid, k: keys[ kid ] };

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