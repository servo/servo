set -e
case $1 in
unit) make check-servo ;;
content) make check-content ;;
ref) make check-ref-cpu ;;
wpt) WPTARGS="--processes=4" make check-wpt ;;
*) echo "Task $1 not enabled for OSX"
esac
