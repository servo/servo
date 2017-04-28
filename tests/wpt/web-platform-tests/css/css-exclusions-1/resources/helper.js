function checkLinePos(spanID,expectedPos,coordToCheck) {
    if(coordToCheck == "top")
        var spanToCheck = document.getElementById(spanID).getBoundingClientRect().top;
    else if(coordToCheck == "right")
        var spanToCheck = document.getElementById(spanID).getBoundingClientRect().right;
    else if(coordToCheck == "left")
        var spanToCheck = document.getElementById(spanID).getBoundingClientRect().left;
    else
        var spanToCheck = document.getElementById(spanID).getBoundingClientRect().bottom;

    /* Verify that the span appears where expected. It should be at expectedPos
        Test will allow 1/4 line height (3px) of leeway for minor spacing differences */
    return( (spanToCheck >= expectedPos) && (spanToCheck <= (expectedPos+3)) )
}
