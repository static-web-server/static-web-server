# Adapted from https://github.com/briansmith/ring/blob/main/mk/cargo.sh

set -eux -o pipefail

OS_NAME=$(uname -s | tr A-Z a-z)

# Avoid putting the Android tools in `$PATH` because there are tools in this
# directory like `clang` that would conflict with the same-named tools that may
# be needed to compile the build script, or to compile for other targets.
if [ -n "${ANDROID_HOME-}" ]; then
  # Keep the next line in sync with the corresponding line in install-build-tools.sh.
  ndk_version=25.2.9519653
  ANDROID_NDK_ROOT=${ANDROID_NDK_ROOT:-${ANDROID_HOME}/ndk/$ndk_version}
fi
if [ -n "${ANDROID_NDK_ROOT-}" ]; then
  android_tools=${ANDROID_NDK_ROOT}/toolchains/llvm/prebuilt/${OS_NAME}-x86_64/bin
fi

for arg in $*; do
  case $arg in
    --target=*)
      target=${arg#*=}
      ;;
    *)
      ;;
  esac
done

case $target in
  aarch64-linux-android)
    export CC_aarch64_linux_android=$android_tools/aarch64-linux-android21-clang
    export AR_aarch64_linux_android=$android_tools/llvm-ar
    export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$android_tools/aarch64-linux-android21-clang
    ;;
  # aarch64-unknown-linux-gnu)
  #   export CC_aarch64_unknown_linux_gnu=clang-$llvm_version
  #   export AR_aarch64_unknown_linux_gnu=llvm-ar-$llvm_version
  #   export CFLAGS_aarch64_unknown_linux_gnu="--sysroot=/usr/aarch64-linux-gnu"
  #   export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
  #   export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER="$qemu_aarch64"
  #   ;;
  # aarch64-unknown-linux-musl)
  #   export CC_aarch64_unknown_linux_musl=clang-$llvm_version
  #   export AR_aarch64_unknown_linux_musl=llvm-ar-$llvm_version
  #   export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="$rustflags_self_contained"
  #   export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUNNER="$qemu_aarch64"
  #   ;;
  # arm-unknown-linux-gnueabihf)
  #   export CC_arm_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc
  #   export AR_arm_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc-ar
  #   export CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
  #   export CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_RUNNER="$qemu_arm"
  #   ;;
  armv7-linux-androideabi)
    export CC_armv7_linux_androideabi=$android_tools/armv7a-linux-androideabi19-clang
    export AR_armv7_linux_androideabi=$android_tools/llvm-ar
    export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER=$android_tools/armv7a-linux-androideabi19-clang
    ;;
  *)
  ;;
esac

cargo "$@"
