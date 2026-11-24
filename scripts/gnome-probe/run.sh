#!/bin/bash
# Checks the GNOME Shell renderer on the gnome-shell this machine (or chroot) has. Run as a normal user:
#   dbus-run-session scripts/gnome-probe/run.sh
# Needs, beforehand, extensions written by `nsh-widgets gnome` for the kinds Dashboard, Tasks, Agenda, Steps,
# Kinds (two timers on the keys start and end, a clock, a rotate, an Open button) and Ghost (no store), all with
# `--group dev.supernsh.widgets.gallery`, and this directory copied to
# ~/.local/share/gnome-shell/extensions/probe@test. It reads /tmp/probe/out.json back and saves screenshots there.
# Use a throwaway HOME: it sets the user's GNOME settings in a keyfile.
unset DISPLAY
export LIBGL_ALWAYS_SOFTWARE=1 GSETTINGS_BACKEND=keyfile DBUS_SYSTEM_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS
rm -rf /tmp/probe && mkdir -p /tmp/probe
E="['probe@test'"
for k in dashboard tasks agenda steps kinds ghost; do E="$E,'dev.supernsh.widgets.gallery.$k@widgets.supernsh.dev'"; done
gsettings set org.gnome.shell disable-user-extensions false
gsettings set org.gnome.shell enabled-extensions "$E]"
gsettings set org.gnome.shell welcome-dialog-last-shown-version 99
gsettings set org.gnome.desktop.interface color-scheme default
gnome-shell --headless --wayland --no-x11 --virtual-monitor 1000x640 > /tmp/probe/shell.log 2>&1 &
P=$!
for _ in $(seq 70); do
  sleep 1
  [ -f /tmp/probe/out.json ] && break
  kill -0 $P 2>/dev/null || break
done
sleep 1
kill $P 2>/dev/null
wait $P 2>/dev/null
grep -E "JS ERROR|disposed|Bail out" /tmp/probe/shell.log && exit 1
cat /tmp/probe/out.json
