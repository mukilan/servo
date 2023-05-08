#!/bin/env bash

VERSION=1.22.1
cd /tmp

curl "https://gstreamer.freedesktop.org/data/pkg/osx/${VERSION}/gstreamer-1.0-${VERSION}-universal.pkg" -o gstreamer.pkg
curl "https://gstreamer.freedesktop.org/data/pkg/osx/${VERSION}/gstreamer-1.0-devel-${VERSION}-universal.pkg" -o gstreamer-dev.pkg

sudo installer -pkg 'gstreamer.pkg' -target /
sudo installer -pkg 'gstreamer-dev.pkg' -target /
