if ( typeof TextEncoder !== "undefined" && typeof TextDecoder !== "undefined" )
{
    utf8encoder = new TextEncoder('utf-8');
    utf8decoder = new TextDecoder('utf-8');
}
else
{
    utf8encoder = { encode: function( text )
    {
        var result = new Uint8Array(text.length);
        for(var i = 0; i < text.length; i++) { result[i] = text.charCodeAt(i); }
        return result;
    } };

    utf8decoder = { decode: function( buffer )
    {
        return String.fromCharCode.apply(null, new Uint8Array(buffer));
    } };
}

toUtf8 = function( o ) { return utf8encoder.encode( JSON.stringify( o ) ); }
fromUtf8 = function( t ) { return JSON.parse( utf8decoder.decode( t ) ); }