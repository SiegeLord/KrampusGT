#!/bin/bash
set -e
cp target/release/KrampusGT.exe .
magick convert raw/icon.png -define icon:auto-resize:256,128,96,64,48,32,16 -compress zip raw/icon.ico
winpty ../ResourceHacker/ResourceHacker.exe -open KrampusGT.exe -save KrampusGT.exe -action addoverwrite -res raw/icon.ico -mask ICONGROUP,MAINICON,
