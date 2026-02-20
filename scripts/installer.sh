#!/bin/sh

# SWS installer script which does platform detection,
# downloads the corresponding pre-compiled binary and runs it.
# 
# Usage:
# curl --proto '=https' --tlsv1.2 -sSfL https://get.static-web-server.net | sh
# 
# Script adapted from https://github.com/rust-lang/rustup/blob/master/rustup-init.sh
# 

# It runs on Unix shells like {a,ba,da,k,z}sh. It uses the common `local`
# extension. Note: Most shells limit `local` to 1 var per line, contra bash.

if [ "$KSH_VERSION" = 'Version JM 93t+ 2010-03-05' ]; then
    # The version of ksh93 that ships with many illumos systems does not
    # support the "local" extension. Print a message rather than fail in
    # subtle ways later on:
    echo 'SWS installer does not work with this ksh93 version; please try bash!' >&2
    exit 1
fi

set -ue

# SWS latest version
version=${SWS_INSTALL_VERSION:-"2.41.0"}

# Default directory where SWS will be installed
install_dir=${SWS_INSTALL_DIR:-"/usr/local/bin"}

main() {
    need_cmd uname
    need_cmd chmod
    need_cmd rm

    get_architecture || return 1

    local _arch="$RETVAL"
    assert_nz "$_arch" "arch"

    case "$_arch" in
        *windows*)
            echo "For install SWS on Windows, please follow https://static-web-server.net/download-and-install/#windows" 1>&2
            exit 1
    esac

    local _ansi_escapes_are_valid=false
    if [ -t 2 ]; then
        if [ "${TERM+set}" = 'set' ]; then
            case "$TERM" in
                xterm*|rxvt*|urxvt*|linux*|vt*)
                    _ansi_escapes_are_valid=true
                ;;
            esac
        fi
    fi

    if $_ansi_escapes_are_valid; then
        printf "\33[1mStatic Web Server Installer\33[0m\n" 1>&2
    else
        printf "Static Web Server Installer\n" 1>&2
    fi

    info "platform '$_arch' supported"

    if [ ! -d "$install_dir" ]; then
        err "install directory '$install_dir' does not exist, is inaccessible or is not a directory"
    fi

    info "downloading 'static-web-server' (v$version) pre-compiled binary..."

    local _filename="static-web-server-v$version-$_arch"
    local _tarball="$_filename.tar.gz"
    local _url="https://github.com/static-web-server/static-web-server/releases/download/v$version/$_tarball"

    download_file=$(mktemp -d -t sws.XXXXXXX) || err "unable to create sws temporary directory"
    trap 'rm -rf -- "$download_file"' EXIT

    ensure download $_url $download_file/$_tarball

    err=$(tar zxf $download_file/$_tarball -C "$download_file/" --strip-components 1 "$_filename/static-web-server" 2>&1)

    if [ -n "$err" ]; then
        echo "$err" >&2
        if echo "$err" | grep -q 404$; then
            err "pre-compile binary for platform '$_arch' not found, this may be unsupported"
        fi
    fi

    info "installing SWS pre-compiled binary to '$install_dir'..."

    local _sudo=""
    if [ ! -w "$install_dir" ]; then
        _sudo=sudo
        warn "Can not install to '$install_dir' due to insufficient permissions, trying with sudo..."
    fi

    local _sws_bin_path="$install_dir/static-web-server"
    ensure $_sudo install -D -m755 "$download_file/static-web-server" $install_dir
    info "pre-compiled binary installed on '$_sws_bin_path'"

    local _sws_bin_symlink_path="$install_dir/sws"

    if [ -L "$_sws_bin_symlink_path" ]; then
        if [ "$(readlink -- "$_sws_bin_symlink_path")" = "$_sws_bin_path" ]; then
            ensure $_sudo unlink "$_sws_bin_symlink_path"
            ensure $_sudo ln -s $_sws_bin_path "$_sws_bin_symlink_path"
            info "symlink binary created on '$_sws_bin_symlink_path'"
        fi
    else
        ensure $_sudo ln -s $_sws_bin_path "$_sws_bin_symlink_path"
        info "symlink binary created on '$_sws_bin_symlink_path'"
    fi

    local _status=$?

    printf "\n" 1>&2
    printf '%s\n' "$_sws_bin_path --version" 1>&2
    ensure $_sws_bin_path --version
    printf "\n" 1>&2

    info "SWS was installed successfully!"
    info "To uninstall SWS, just remove it from its location."
    info "Visit https://static-web-server.net/getting-started for more details."

    return "$_status"
}

download() {
    if [ -x "$(command -v curl)" ]; then
        curl --proto '=https' --tlsv1.2 -sSf --location $1 --output $2
    else
        if [ ! -x "$(command -v wget)" ]; then
            err "SWS installer requires either 'curl' or 'wget' to be installed but neither was found. Please install one and try again."
        fi

        if [ "$(wget -V 2>&1|head -2|tail -1|cut -f1 -d" ")" = "BusyBox" ]; then
            err "can not use BusyBox 'wget' to download as it can be potentially less secure, please install GNU 'wget' or 'curl' and try again."
        else
            wget --https-only --secure-protocol=TLSv1_2 -qO- $1 > $2
        fi
    fi
}

check_proc() {
    # Check for /proc by looking for the /proc/self/exe link
    # This is only run on Linux
    if ! test -L /proc/self/exe ; then
        err "fatal: Unable to find /proc/self/exe.  Is /proc mounted?  Installation cannot proceed without /proc."
    fi
}

get_bitness() {
    need_cmd head
    # Architecture detection without dependencies beyond coreutils.
    # ELF files start out "\x7fELF", and the following byte is
    #   0x01 for 32-bit and
    #   0x02 for 64-bit.
    # The printf builtin on some shells like dash only supports octal
    # escape sequences, so we use those.
    local _current_exe_head
    _current_exe_head=$(head -c 5 /proc/self/exe )
    if [ "$_current_exe_head" = "$(printf '\177ELF\001')" ]; then
        echo 32
    elif [ "$_current_exe_head" = "$(printf '\177ELF\002')" ]; then
        echo 64
    else
        err "unknown platform bitness"
    fi
}

is_host_amd64_elf() {
    need_cmd head
    need_cmd tail
    # ELF e_machine detection without dependencies beyond coreutils.
    # Two-byte field at offset 0x12 indicates the CPU,
    # but we're interested in it being 0x3E to indicate amd64, or not that.
    local _current_exe_machine
    _current_exe_machine=$(head -c 19 /proc/self/exe | tail -c 1)
    [ "$_current_exe_machine" = "$(printf '\076')" ]
}

get_endianness() {
    local cputype=$1
    local suffix_eb=$2
    local suffix_el=$3

    # detect endianness without od/hexdump, like get_bitness() does.
    need_cmd head
    need_cmd tail

    local _current_exe_endianness
    _current_exe_endianness="$(head -c 6 /proc/self/exe | tail -c 1)"
    if [ "$_current_exe_endianness" = "$(printf '\001')" ]; then
        echo "${cputype}${suffix_el}"
    elif [ "$_current_exe_endianness" = "$(printf '\002')" ]; then
        echo "${cputype}${suffix_eb}"
    else
        err "unknown platform endianness"
    fi
}

get_architecture() {
    local _ostype _cputype _bitness _arch _clibtype
    _ostype="$(uname -s)"
    _cputype="$(uname -m)"
    _clibtype="gnu"

    if [ "$_ostype" = Linux ]; then
        if [ "$(uname -o)" = Android ]; then
            _ostype=Android
        fi
        if ldd --version 2>&1 | grep -q 'musl'; then
            _clibtype="musl"
        fi
    fi

    if [ "$_ostype" = Darwin ] && [ "$_cputype" = i386 ]; then
        # Darwin `uname -m` lies
        if sysctl hw.optional.x86_64 | grep -q ': 1'; then
            _cputype=x86_64
        fi
    fi

    if [ "$_ostype" = SunOS ]; then
        # Both Solaris and illumos presently announce as "SunOS" in "uname -s"
        # so use "uname -o" to disambiguate.  We use the full path to the
        # system uname in case the user has coreutils uname first in PATH,
        # which has historically sometimes printed the wrong value here.
        if [ "$(/usr/bin/uname -o)" = illumos ]; then
            _ostype=illumos
        fi

        # illumos systems have multi-arch userlands, and "uname -m" reports the
        # machine hardware name; e.g., "i86pc" on both 32- and 64-bit x86
        # systems.  Check for the native (widest) instruction set on the
        # running kernel:
        if [ "$_cputype" = i86pc ]; then
            _cputype="$(isainfo -n)"
        fi
    fi

    case "$_ostype" in

        Android)
            _ostype=linux-android
            ;;

        Linux)
            check_proc
            _ostype=unknown-linux-$_clibtype
            _bitness=$(get_bitness)
            ;;

        FreeBSD)
            _ostype=unknown-freebsd
            ;;

        NetBSD)
            _ostype=unknown-netbsd
            ;;

        # DragonFly)
        #     _ostype=unknown-dragonfly
        #     ;;

        Darwin)
            _ostype=apple-darwin
            ;;

        illumos)
            _ostype=unknown-illumos
            ;;

        MINGW* | MSYS* | CYGWIN* | Windows_NT)
            _ostype=pc-windows-gnu
            ;;

        *)
            err "unsupported OS type: $_ostype"
            ;;

    esac

    case "$_cputype" in
        i386 | i486 | i686 | i786 | x86)
            _cputype=i686
            ;;

        xscale | arm)
            _cputype=arm
            if [ "$_ostype" = "linux-android" ]; then
                _ostype=linux-androideabi
            fi
            ;;

        armv6l)
            _cputype=arm
            if [ "$_ostype" = "linux-android" ]; then
                _ostype=linux-androideabi
            else
                _ostype="${_ostype}eabihf"
            fi
            ;;

        armv7l | armv8l)
            _cputype=armv7
            if [ "$_ostype" = "linux-android" ]; then
                _ostype=linux-androideabi
            else
                _ostype="${_ostype}eabihf"
            fi
            ;;

        aarch64 | arm64)
            _cputype=aarch64
            ;;

        x86_64 | x86-64 | x64 | amd64)
            _cputype=x86_64
            ;;

        *)
            err "unknown CPU type: $_cputype"

    esac

    # Detect 64-bit linux with 32-bit userland
    if [ "${_ostype}" = unknown-linux-gnu ] && [ "${_bitness}" -eq 32 ]; then
        case $_cputype in
            x86_64)
                if [ -n "${RUSTUP_CPUTYPE:-}" ]; then
                    _cputype="$RUSTUP_CPUTYPE"
                else {
                    # 32-bit executable for amd64 = x32
                    if is_host_amd64_elf; then {
                        echo "This host is running an x32 userland; x32 is not supported" 1>&2
                        exit 1
                    }; else
                        _cputype=i686
                    fi
                }; fi
                ;;
            # mips64)
            #     _cputype=$(get_endianness mips '' el)
            #     ;;
            # powerpc64)
            #     _cputype=powerpc
            #     ;;
            aarch64)
                _cputype=armv7
                if [ "$_ostype" = "linux-android" ]; then
                    _ostype=linux-androideabi
                    err "unsupported OS type: $_ostype"
                else
                    _ostype="${_ostype}eabihf"
                fi
                ;;
            # riscv64gc)
            #     err "riscv64 with 32-bit userland unsupported"
            #     ;;
        esac
    fi

    # Detect armv7 but without the CPU features Rust needs in that build,
    # and fall back to arm.
    # See https://github.com/rust-lang/rustup.rs/issues/587.
    if [ "$_ostype" = "unknown-linux-gnueabihf" ] && [ "$_cputype" = armv7 ]; then
        if ensure grep '^Features' /proc/cpuinfo | grep -q -v neon; then
            # At least one processor does not have NEON.
            _cputype=arm
        fi
    fi

    _arch="${_cputype}-${_ostype}"

    RETVAL="$_arch"
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
    if $_ansi_escapes_are_valid; then
        printf '\33[1m%s:\33[0m %s\n' "$1" "$2" >&2
    else
        printf '%s: %s\n' "$1" "$2" >&2
    fi
}


need_cmd() {
    if ! check_cmd "$1"; then
        err "need '$1' (command not found)"
    fi
}

check_cmd() {
    command -v "$1" > /dev/null 2>&1
}

assert_nz() {
    if [ -z "$1" ]; then err "assert_nz $2"; fi
}

# Run a command that should never fail. If the command fails execution
# will immediately terminate with an error showing the failing
# command.
ensure() {
    if ! "$@"; then err "command failed: $*"; fi
}

main "$@" || exit 1
