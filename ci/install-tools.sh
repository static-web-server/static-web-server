# Adapted from https://github.com/briansmith/ring/blob/main/mk/install-build-tools.sh

set -eux -o pipefail

target=$1

case $target in
--target*android*)
  # https://blog.rust-lang.org/2023/01/09/android-ndk-update-r25.html says
  # "Going forward the Android platform will target the most recent LTS NDK,
  # allowing Rust developers to access platform features sooner. These updates
  # should occur yearly and will be announced in release notes." Assume that
  # means that we should always prefer to be using the latest 25.x.y version of
  # the NDK until the Rust project announces that we should use a higher major
  # version number.
  #
  # TODO: This should probably be implemented as a map of Rust toolchain version
  # to NDK version; e.g. our MSRV might (only) support an older NDK than the
  # latest stable Rust toolchain.
  #
  # Keep the following line in sync with the corresponding line in cargo.sh.
  ndk_version=25.2.9519653

  mkdir -p "${ANDROID_HOME}/licenses"
  android_license_file="${ANDROID_HOME}/licenses/android-sdk-license"
  accept_android_license=24333f8a63b6825ea9c5514f83c2829b004d1fee
  grep --quiet --no-messages "$accept_android_license" "$android_license_file" \
    || echo $accept_android_license  >> "$android_license_file"
  "${ANDROID_HOME}/cmdline-tools/latest/bin/sdkmanager" "ndk;$ndk_version"

  # XXX: Older Rust toolchain versions link with `-lgcc` instead of `-lunwind`;
  # see https://github.com/rust-lang/rust/pull/85806.
  find -L ${ANDROID_NDK_ROOT:-${ANDROID_HOME}/ndk/$ndk_version} -name libunwind.a \
          -execdir sh -c 'echo "INPUT(-lunwind)" > libgcc.a' \;
  ;;
esac
