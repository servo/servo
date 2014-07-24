sudo add-apt-repository -y ppa:ubuntu-toolchain-r/test
sudo apt-key adv --recv-keys --keyserver keyserver.ubuntu.com 1E9377A2BA9EF27F
sudo apt-get update -qq
echo showing holds
apt-mark showhold 
sudo apt-get install -qq --force-yes -y autoconf2.13 gperf libxxf86vm-dev libglfw-dev libstdc++6-4.7-dev libedit-dev ia32-libs ia32-libs-multiarch
echo showing holds x2
apt-mark showhold 
sudo apt-get install gcc-4.7
sudo update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-4.7 50
sudo apt-get install g++-4.7
sudo update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-4.7 50
wget http://servo-rust.s3.amazonaws.com/llvm-for-rustc2/llvm-for-rustc-linux.tgz
tar zxvf llvm-for-rustc-linux.tgz

# Android SDK - will need for testing later
#wget http://dl.google.com/android/android-sdk_r22.3-linux.tgz
#tar -zxf android-sdk_r22.3-linux.tgz
#export ANDROID_HOME=`pwd`/android-sdk-linux
#export PATH=${PATH}:${ANDROID_HOME}/tools:${ANDROID_HOME}/platform-tools

# newest Android NDK
if [ `uname -m` = x86_64]; then wget http://dl.google.com/android/ndk/android-ndk-r9c-linux-x86_64.tar.bz2 -O ndk.tgz; else wget http://dl.google.com/android/ndk/android-ndk-r9c-linux-x86.tar.bz2 -O ndk.tgz; fi
tar -zxvf ndk.tgz
`pwd`/android-ndk-r9c/build/tools/make-standalone-toolchain.sh --platform=android-14 --install-dir=`pwd`/ndk_standalone --ndk-dir=`pwd`/android-ndk-r9c
export ANDROID_NDK_HOME=`pwd`/android-ndk-r9c
export PATH=${PATH}:${ANDROID_HOME}/tools:${ANDROID_HOME}/platform-tools:${ANDROID_NDK_HOME}

# Potentially also update Android SDK tools & variables
#echo "sdk.dir=$ANDROID_HOME" > local.properties
#echo yes | android update sdk -a -t tools,platform-tools,extra-android-support,extra-android-m2repository,android-19,build-tools-19.0.1,extra-google-google_play_services,extra-google-m2repository --force --no-ui
