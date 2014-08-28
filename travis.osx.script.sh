set -e
case $1 in
unit) ./mach test-unit ;;
content) ./mach test-content ;;
ref) ./mach test-ref --kind cpu ;;
wpt1) ./mach test-wpt --processes=4 --total-chunks=2 --this-chunk=1 ;;
wpt2) ./mach test-wpt --processes=4 --total-chunks=2 --this-chunk=2 ;;
*) echo "Task $1 not enabled for OSX"
esac
