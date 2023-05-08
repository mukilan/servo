#!/bin/env bash

VERSION=1.22.1
URL_BASE=https://github.com/servo/servo-build-deps/releases/download/macOS

cd /tmp
curl "${URL_BASE}/gstreamer-1.0-${VERSION}-universal.pkg" -o gstreamer.pkg
curl "${URL_BASE}/gstreamer-1.0-devel-${VERSION}-universal.pkg" -o gstreamer-dev.pkg
sudo installer -pkg 'gstreamer.pkg' -target /
sudo installer -pkg 'gstreamer-dev.pkg' -target /
