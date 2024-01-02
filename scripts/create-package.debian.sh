#!/bin/bash

set -Cu
set -Ee
set -o pipefail
shopt -s nullglob

stime=$(date +%Y%m%d%H%M%S%Z)
based=$(readlink -f $(dirname $0)/..)
pname=$(basename $0)

exec 3>&2
# logd=$based/log
# exec 3>&2 2>$logd/$pname.$stime.$$.log
# set -vx

MSG() {
    echo "$pname pid:$$ stime:$stime etime:$(date +%Y%m%d%H%M%S%Z) $@"	>&3
}

tmpd=$(mktemp -d -t "$pname.$stime.$$.XXXXXXXX")/
if [ 0 -ne "$?" ] ; then
    MSG "line:$LINENO FATAL can not make temporally directory."
    exit 1
fi

trap 'BEFORE_EXIT' EXIT
BEFORE_EXIT()	{
    rm -rf $tmpd
}

trap 'ERROR_HANDLER' ERR
export EMSG="line:$LINENO ERROR"
ERROR_HANDLER()	{
    MSG "line:$LINENO ERROR status ${PIPESTATUS[@]}"
    [ "$EMSG" ] && MSG $EMSG
    touch $tmpd/ERROR	# for child process error detection
    MSG "line:$LINENO EXIT with error."
    exit 1		# root process trigger BEFORE_EXIT function
}

PROGRESS() {
    lineno="$1"
    shift
    PMSG="$*"
    MSG "line:$lineno INFO $PMSG"
    EMSG="line:$lineno ERROR while $PMSG"
}

################################################################
PROGRESS "$LINENO" "reading arguments"

PG_VERSION=$1
ARCH=$2

EXT_NAME=$(yq -r -o json '.package.name' Cargo.toml)
EXT_VERSION=$(yq -r -o json '.package.version' Cargo.toml)
PKG_NAME=$(echo $EXT_NAME | tr '_' '-')

BLD_BASED=$based/target/release/${EXT_NAME}-pg${PG_VERSION}
PKG_BASED=${BLD_BASED}.debian_package_tmp

cd $based
id

################################################################
PROGRESS "$LINENO" "building binaries"
# selects the pgVer from pg_config on path
# https://github.com/tcdi/pgrx/issues/288
cargo pgrx package --no-default-features --features "pg${PG_VERSION}"

################################################################
PROGRESS "$LINENO" "cleanup package directory"
sudo rm -rf ${PKG_BASED}
cp -rp ${BLD_BASED} ${PKG_BASED}

PROGRESS "$LINENO" "building installable package"
mkdir -p ${PKG_BASED}/DEBIAN
rm -f ${PKG_BASED}/DEBIAN/control
cat <<EOF   |
Package: ${PKG_NAME}
Version: ${EXT_VERSION}
Architecture: ${ARCH}
Maintainer: Nakamura Kazutaka
Description: A PostgreSQL extension for UUIDv7
EOF
tee ${PKG_BASED}/DEBIAN/control >&3

PROGRESS "$LINENO" "copying dynamic libraries"
mkdir -p ${PKG_BASED}/usr/lib/postgresql/lib
pushd ${PKG_BASED}/usr/lib/postgresql/lib   > /dev/null
rm -f ${EXT_NAME}.so
cp -s ../${PG_VERSION}/lib/${EXT_NAME}.so .
popd   > /dev/null

PROGRESS "$LINENO" "copying extension files"
mkdir -p ${PKG_BASED}/var/lib/postgresql/extension
pushd ${PKG_BASED}/var/lib/postgresql/extension   > /dev/null
rm -f ${EXT_NAME}.control
cp -s ../../../../usr/share/postgresql/${PG_VERSION}/extension/${EXT_NAME}.control .
rm -f ${EXT_NAME}--${EXT_VERSION}.sql
cp -s ../../../../usr/share/postgresql/${PG_VERSION}/extension/${EXT_NAME}--${EXT_VERSION}.sql .
popd   > /dev/null

PROGRESS "$LINENO" "creating deb package"
chmod -R 00755 ${PKG_BASED}
sudo chown -R root:root ${PKG_BASED}
dpkg-deb -Zxz --build --root-owner-group ${PKG_BASED}

# TODO artifact name shoud be changed
PKG_OUT=$based/target/${PKG_NAME}-${PG_VERSION}-${ARCH}-linux-gnu.deb
PROGRESS "$LINENO" "copying package to $PKG_OUT"
mv ${PKG_BASED}.deb $PKG_OUT

################################################################
PROGRESS "$LINENO" "exiting"
shopt -u nullglob
MSG "line:$LINENO EXIT without error."
exit 0
