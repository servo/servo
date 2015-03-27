onmessage = function(evt)
{
    for (var i=0; true; i++)
    {
        if (i%1000 == 1)
        {
            postMessage(i);
        }
    }
}