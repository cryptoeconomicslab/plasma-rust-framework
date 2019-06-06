#!/bin/bash

FILE=$HOME/.NDK/arm64
if test -d "$FILE"; then
    echo "$FILE exist"
else
    wget -nv https://dl.google.com/android/repository/android-ndk-r19c-linux-x86_64.zip
    unzip -qq android-ndk-r19c-linux-x86_64.zip -d $HOME
    export ANDROID_NDK_HOME=$HOME/android-ndk-r19c
    export PATH=$PATH:$ANDROID_NDK_HOME
    mkdir "$ANDROID_HOME/licenses" || true
    echo -e "\n8933bad161af4178b1185d1a37fbf41ea5269c55" > "$ANDROID_HOME/licenses/android-sdk-license"
    echo -e "\n84831b9409646a918e30573bab4c9c91346d8abd" > "$ANDROID_HOME/licenses/android-sdk-preview-license"
    rm android-ndk-r19c-linux-x86_64.zip
    mkdir $HOME/.NDK
    ${ANDROID_NDK_HOME}/build/tools/make_standalone_toolchain.py --api 28 --arch arm64 --install-dir $HOME/.NDK/arm64;
    ${ANDROID_NDK_HOME}/build/tools/make_standalone_toolchain.py --api 28 --arch arm --install-dir $HOME/.NDK/arm;
    ${ANDROID_NDK_HOME}/build/tools/make_standalone_toolchain.py --api 28 --arch x86 --install-dir $HOME/.NDK/x86;
fi
