function xvfb_start() {
    GEOMETRY="$SCREEN_WIDTH""x""$SCREEN_HEIGHT""x""$SCREEN_DEPTH"
    xvfb-run --server-args="-screen 0 $GEOMETRY -ac +extension RANDR" $@
}
