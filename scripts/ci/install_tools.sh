# Installer script for build tools needed by CI jobs.
# It is intended to be run on GitHub Actions hosted runners.
# USAGE: scripts/ci/install_tools.sh --target=<target> --build=<build>
#
# Script adapted from https://github.com/briansmith/ring/blob/main/mk/install-build-tools.sh

set -eux -o pipefail

main() {
  # 1. Initialize variables with default values (optional)
  local TARGET=""
  local BUILD=""
  local ARCH=$(uname -m)
  local ansi_escapes_valid=false

  # 2. Detect if ANSI escape codes are valid in the current terminal.
  detect_ansi

  # 3. Parse arguments in the form of --key=value
  parse_args "$@"

  # 4. Run the appropriate setup function based on the parsed arguments.
  setup_all
}

detect_ansi() {
  if [ -t 2 ]; then
    if [ "${TERM+set}" = 'set' ]; then
      case "$TERM" in
        xterm*|rxvt*|urxvt*|linux*|vt*)
          ansi_escapes_valid=true
        ;;
      esac
    fi
  fi
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --target=*)
        TARGET="${1#*=}"
        shift
        ;;
      --build=*)
        BUILD="${1#*=}"
        shift
        ;;
      *)
        err "Cannot parse argument: $1"
        ;;
    esac
  done
}

setup_all() {
  info "Installing build tools for '$TARGET' and '$BUILD' on '$ARCH'..."

  # a. BUILD cases (prioritized over 'TARGET' cases) / --build=${{ matrix.build }}
  case $BUILD in
    # Linux GNU 32-bit
    i686-unknown-linux-gnu)
      setup_linux_gnu_32_bit
      exit 0
      ;;

    # FIPS
    *fips)
      setup_linux_fips_64_bit
      exit 0
      ;;
    *)
      info "No build case matching, nothing to do."
      ;;
  esac

  # b. TARGET cases / --target=${{ matrix.target }}
  case $TARGET in
    # Android
    *android*)
      setup_linux_android
      exit 0
      ;;
    # Linux amd64/arm64 (GNU and musl)
    aarch64-unknown-linux-gnu|aarch64-unknown-linux-musl|x86_64-unknown-linux-gnu|x86_64-unknown-linux-musl)
      echo "Setting up build tools for Linux 64-bit on '$ARCH'..."
      setup_linux_64_bit
      exit 0
      ;;
    *)
      info "No target case matching, nothing to do."
      ;;
  esac
}

setup_linux_fips_64_bit() {
  setup_linux_64_bit

  info "Setting up Cargo features for FIPS builds..."
  echo "CARGO_FEATURES=--no-default-features --features all-fips" >> $GITHUB_ENV
  echo "RUSTFLAGS=--cfg tokio_unstable" >> $GITHUB_ENV
  info "Successfully configured!"
}

# Linux 64-bit (arm64/amd64) and (GNU/musl) builds.
setup_linux_64_bit() {
  info "Installing build tools for Linux 64-bit on '$ARCH'..."

  ensure sudo apt-get update
  ensure sudo apt-get install -y \
    cmake \
    golang \
    libclang-dev \
    musl-tools \
    musl-dev \
    linux-headers-generic

  info "Configuring musl environment for $ARCH"

  # Dynamically target Ubuntu's architecture-isolated musl include tree
  MUSL_INC_DIR="/usr/include/${ARCH}-linux-musl"
  ensure sudo mkdir -p "$MUSL_INC_DIR"

  # 1. Link the universal Linux kernel headers into the architecture's musl path
  ensure sudo ln -sf /usr/include/linux "$MUSL_INC_DIR/linux"
  ensure sudo ln -sf /usr/include/asm-generic "$MUSL_INC_DIR/asm-generic"

  # 2. Link architecture-specific headers and compiler wrappers
  case "$ARCH" in
    aarch64)
      ensure sudo ln -sf /usr/include/aarch64-linux-gnu/asm "$MUSL_INC_DIR/asm"
      ensure sudo ln -sf /usr/bin/musl-gcc /usr/local/bin/aarch64-linux-musl-gcc
      ensure sudo ln -sf /usr/bin/g++ /usr/local/bin/aarch64-linux-musl-g++
      info "Successfully configured for aarch64."
      ;;
    x86_64)
      ensure sudo ln -sf /usr/include/x86_64-linux-gnu/asm "$MUSL_INC_DIR/asm"
      ensure sudo ln -sf /usr/bin/musl-gcc /usr/local/bin/x86_64-linux-musl-gcc
      ensure sudo ln -sf /usr/bin/g++ /usr/local/bin/x86_64-linux-musl-g++
      info "Successfully configured for x86_64."
      ;;
    *)
      ensure sudo rm -rf $MUSL_INC_DIR
      err "Unsupported runner architecture: $ARCH"
      ;;
  esac
}

setup_linux_gnu_32_bit() {
  info "Installing build tools for Linux GNU 32-bit on '$ARCH'..."

  ensure sudo apt-get update
  ensure sudo apt-get install -y gcc-multilib g++-multilib
  info "Successfully configured!"
}

setup_linux_android() {
  info "Installing build tools for Android on '$ARCH'..."

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

  ensure mkdir -p "${ANDROID_HOME}/licenses"
  android_license_file="${ANDROID_HOME}/licenses/android-sdk-license"
  accept_android_license=24333f8a63b6825ea9c5514f83c2829b004d1fee
  grep --quiet --no-messages "$accept_android_license" "$android_license_file" \
    || echo $accept_android_license  >> "$android_license_file"
  ensure "${ANDROID_HOME}/cmdline-tools/latest/bin/sdkmanager" "ndk;$ndk_version"

  # XXX: Older Rust toolchain versions link with `-lgcc` instead of `-lunwind`;
  # see https://github.com/rust-lang/rust/pull/85806.
  ensure find -L ${ANDROID_NDK_ROOT:-${ANDROID_HOME}/ndk/$ndk_version} -name libunwind.a \
    -execdir sh -c 'echo "INPUT(-lunwind)" > libgcc.a' \;
}

info() {
  __print 'info' "$1" >&2
}

warn() {
  __print 'warn' "$1" >&2
}

err() {
  __print 'error' "$1" >&2
  exit 1
}

__print() {
  if $ansi_escapes_valid; then
    printf '\33[1m%s:\33[0m %s\n' "$1" "$2" >&2
  else
    printf '%s: %s\n' "$1" "$2" >&2
  fi
}

# Run a command that should never fail. If the command fails execution
# will immediately terminate with an error showing the failing
# command.
ensure() {
    if ! "$@"; then err "command failed: $*"; fi
}

main "$@" || exit 1
