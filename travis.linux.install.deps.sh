sudo add-apt-repository ppa:ubuntu-toolchain-r/test -y
sudo apt-get update -q
sudo apt-get install -qq --force-yes -y xserver-xorg-input-void xserver-xorg-video-dummy xpra
sudo apt-get install -qq --force-yes -y autoconf2.13 gperf libxxf86vm-dev libglfw-dev libstdc++6-4.7-dev
echo ttf-mscorefonts-installer msttcorefonts/accepted-mscorefonts-eula select true | sudo debconf-set-selections
sudo apt-get install ttf-mscorefonts-installer > /dev/null
