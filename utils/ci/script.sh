# Derived from https://github.com/japaric/trust

set -ex

main() {
    cross test --target $TARGET
    cross test --target $TARGET --benches
    cross test --target $TARGET --examples
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
