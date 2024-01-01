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
EXT_NAME=$2
EXT_VERSION=$3
ARCH=$4
PKG_NAME=$5

cd $based

################################################################
PROGRESS "$LINENO" "building binaries"
# selects the pgVer from pg_config on path
# https://github.com/tcdi/pgrx/issues/288
cargo pgrx package --no-default-features --features "pg${PG_VERSION}"

################################################################
PROGRESS "$LINENO" "building installable package"
mkdir -p $tmpd/package/DEBIAN
cat <<EOF   > $tmpd/package/DEBIAN/control
Package: ${PKG_NAME}
Version: ${EXT_VERSION}
Architecture: ${ARCH}
Maintainer: Nakamura Kazutaka
Description: A PostgreSQL extension for UUIDv7
EOF

PROGRESS "$LINENO" "copying dynamic libraries"
mkdir -p $tmpd/package/usr/lib/postgresql/lib
cp  $based/target/release/${EXT_NAME}-pg${PG_VERSION}/usr/lib/postgresql/${PG_VERSION}/lib/${EXT_NAME}.so   \
    $tmpd/package/usr/lib/postgresql/lib
mkdir -p $tmpd/package/usr/lib/postgresql/${PG_VERSION}/lib
cp -s $tmpd/package/usr/lib/postgresql/lib/*.so $tmpd/package/usr/lib/postgresql/${PG_VERSION}/lib

PROGRESS "$LINENO" "copying extension files"
mkdir -p $tmpd/package/var/lib/postgresql/extension
cp  $based/target/release/${EXT_NAME}-pg${PG_VERSION}/usr/share/postgresql/${PG_VERSION}/extension/${EXT_NAME}.control   \
    $based/target/release/${EXT_NAME}-pg${PG_VERSION}/usr/share/postgresql/${PG_VERSION}/extension/${EXT_NAME}--${EXT_VERSION}.sql   \
    $tmpd/package/var/lib/postgresql/extension
mkdir -p $tmpd/package/usr/share/postgresql/${PG_VERSION}/extension
cp -s   \
    $tmpd/package/var/lib/postgresql/extension/${EXT_NAME}.control  \
    $tmpd/package/var/lib/postgresql/extension/${EXT_NAME}--${EXT_VERSION}.sql  \
    $tmpd/package/usr/share/postgresql/${PG_VERSION}/extension


PROGRESS "$LINENO" "creating deb package"
chmod -R 00755 $tmpd/package
# chown -R root:root $tmpd/package
dpkg-deb -Zxz --build --root-owner-group $tmpd/package

PKG_OUT=$based/target/${EXT_NAME}-${PG_VERSION}-${ARCH}-linux-gnu.deb
PROGRESS "$LINENO" "copying package to $PKG_OUT"
mv $tmpd/package.deb $PKG_OUT

################################################################
PROGRESS "$LINENO" "exiting"
shopt -u nullglob
MSG "line:$LINENO EXIT without error."
exit 0
