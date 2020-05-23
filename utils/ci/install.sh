# From https://github.com/japaric/trust

set -ex

main() {
    local host=
    if [ $TRAVIS_OS_NAME = linux ]; then
        host=x86_64-unknown-linux-musl
    else
        host=x86_64-apple-darwin
    fi

    # Builds for iOS are done on OSX, but require the specific target to be
    # installed.
    case $TARGET in
        aarch64-apple-ios)
            rustup target install aarch64-apple-ios
            ;;
        armv7-apple-ios)
            rustup target install armv7-apple-ios
            ;;
        armv7s-apple-ios)
            rustup target install armv7s-apple-ios
            ;;
        i386-apple-ios)
            rustup target install i386-apple-ios
            ;;
        x86_64-apple-ios)
            rustup target install x86_64-apple-ios
            ;;
    esac

    # Pin the Cross version to avoid breaking the CI
    local cross_version="v0.2.0"
    wget -O cross.tar.gz https://github.com/rust-embedded/cross/releases/download/${cross_version}/cross-${cross_version}-${host}.tar.gz
    tar -xzf cross.tar.gz
    rm -f cross.tar.gz
    mv ./cross $HOME/.cargo/bin
}

main
