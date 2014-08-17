set -e
case $1 in
unit) make check-servo ;;
content) make check-content ;;
ref) make check-ref-cpu ;;
wpt1) WPTARGS="--processes=4 --total-chunks=2 --this-chunk=1" make check-wpt ;;
wpt2) WPTARGS="--processes=4 --total-chunks=2 --this-chunk=2" make check-wpt ;;
*) echo "Task $1 not enabled for OSX"
esac
