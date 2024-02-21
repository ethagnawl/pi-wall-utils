#!/bin/bash

set -euo pipefail

IP=$1
HOSTNAME=$2

cd ~/Downloads

sudo apt-get update

sudo apt-get install -y libegl1-mesa-dev

wget \
  http://dl.piwall.co.uk/pwlibs1_1.1_armhf.deb \
  http://dl.piwall.co.uk/pwomxplayer_20130815_armhf.deb

sudo dpkg -i \
  pwlibs1_1.1_armhf.deb \
  pwomxplayer_20130815_armhf.deb

wget -O test.mp4 \
  https://download.blender.org/peach/bigbuckbunny_movies/BigBuckBunny_320x180.mp4

# pwomxplayer ./test.mp4

echo "ip route add 224.0.0.4/24 via $IP" | sudo tee -a /lib/dhcpcd/dhcpcd-hooks/40-route

sudo tee -a "/etc/dhcpcd.conf" > /dev/null <<EOF
interface wlan0
static ip_address=$IP/24
static routers=192.168.0.1
static domain_name_servers=8.8.8.8
EOF

echo "$HOSTNAME" | sudo tee /etc/hostname

PI_TILE="/home/pi/.pitile"
touch $PI_TILE
tee $PI_TILE > /dev/null <<EOF
[tile]
id=$HOSTNAME
EOF

# sudo reboot
