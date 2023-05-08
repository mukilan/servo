#!/bin/sh

VERSION=1.22.2
URL_BASE=https://github.com/servo/servo-build-deps/releases/download/macOS

cd /tmp
curl -L "${URL_BASE}/gstreamer-1.0-${VERSION}-universal.pkg" -o gstreamer.pkg
curl -L "${URL_BASE}/gstreamer-1.0-devel-${VERSION}-universal.pkg" -o gstreamer-dev.pkg
sudo installer -pkg 'gstreamer.pkg' -target /
sudo installer -pkg 'gstreamer-dev.pkg' -target /
