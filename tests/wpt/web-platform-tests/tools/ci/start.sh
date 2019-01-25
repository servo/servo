# This script is designed to be sourced from tools/docker/start.sh

# Start userspace OOM killer: https://github.com/rfjakob/earlyoom
# It will report memory usage every minute and prefer to kill browsers.
sudo earlyoom -p -r 60 --prefer '(chrome|firefox)' --avoid 'python' &

sudo sh -c './wpt make-hosts-file >> /etc/hosts'

if [[ $BROWSER == "chrome" ]] || [[ "$BROWSER" == all ]]
then
    # Install Chrome dev
    if [[ "$CHANNEL" == "dev" ]] || [[ "$CHANNEL" == "nightly" ]]
    then
       deb_archive=google-chrome-unstable_current_amd64.deb
    elif [[ "$CHANNEL" == "beta" ]]
    then
        deb_archive=google-chrome-beta_current_amd64.deb
    elif [[ "$CHANNEL" == "stable" ]]
    then
        deb_archive=google-chrome-stable_current_amd64.deb
    else
        echo Unrecognized release channel: $CHANNEL >&2
        exit 1
    fi
    wget https://dl.google.com/linux/direct/$deb_archive

    sudo apt-get -qqy update && sudo gdebi -n $deb_archive
fi

sudo Xvfb $DISPLAY -screen 0 ${SCREEN_WIDTH}x${SCREEN_HEIGHT}x${SCREEN_DEPTH} &
sudo fluxbox -display $DISPLAY &
