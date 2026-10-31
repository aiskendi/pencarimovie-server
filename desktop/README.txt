PencariMovie Server
===================

IT IS ONE BINARY. `pencarimovie-server` (or `pencarimovie-server.exe`) contains
the whole server: the dashboard, the Stremio/Eclipse addon, the Telegram engine
and the caches. Run it and it works:

    ./pencarimovie-server --port 8088        # http://127.0.0.1:8088/
    pencarimovie-server.exe --port 8088

No PHP, no Composer, no vendor/, no public/ folder. `storage/` next to the binary
holds sessions, auth.json and caches - KEEP IT between upgrades.

THE TRAY IS IN THE BINARY. There is no tray script and no tray daemon:

    pencarimovie-server --tray        # icon + server in one process
    pencarimovie-server.exe --tray

Right-click the icon: open the dashboard / add-on page, copy the add-on URL,
open the log, "Run at login", stop the server. The logo is compiled in, so the
icon is there even in a folder that holds nothing but the executable.

Everything else in this folder is an OPTIONAL convenience so the server can be
started without a console and without remembering a command:

  Windows
    start.bat          console launcher (you see the log)
    start-tray.bat     starts hidden with the tray icon (double-click = dashboard)
    autostart-on.bat   start at login (HKCU Run, no admin rights)
    autostart-off.bat  remove that entry
    tray.ps1           compatibility shim only: forwards old login entries that
                       still point at it to `pencarimovie-server.exe --tray`
    pencarimovie.ico   logo, for older installs; the binary has its own copy

  Linux
    start.sh           console launcher (--open also opens the browser)
    start-tray.sh      `pencarimovie-server --tray` (icon needs `yad`, else URL)
    autostart-on.sh    install ~/.config/autostart/pencarimovie-server.desktop
    autostart-off.sh   remove it
    pencarimovie.png   logo, if you want a desktop shortcut icon

  macOS
    start.command      double-click launcher (opens the dashboard)
    autostart-on.sh    LaunchAgent (no window, starts at login)
    autostart-off.sh   remove it
    pencarimovie.png   logo for a shortcut

  (macOS has no shell-level menu-bar API, so there is no tray there without
   shipping a compiled .app bundle. The LaunchAgent covers "always running".)

First run
  1. start the server (any way above)
  2. open http://127.0.0.1:8088/            dashboard
  3. install the add-on from http://127.0.0.1:8088/manifest.json
     (or open http://127.0.0.1:8088/#addon and use the buttons)

LAN / TV access
  The Windows launchers (start.bat, start-tray.bat, pms, the login entry) bind
  all interfaces, so a TV on the same wifi can use the LAN URL they print.
  Everything else (bare binary, Linux and macOS launchers) is loopback-only:
  add --host 0.0.0.0 to reach it from another device.
  Loopback again on Windows:  set PM_LISTEN_HOST=127.0.0.1  (a --host flag wins)
  the dashboard footer / /api/lan-ip show the LAN URL to install on a TV box.

Flags
  --port N         listen port (or env PORT)
  --host ADDR      bind address (or env HOST / PM_LISTEN_HOST)
  --workers N      worker threads
  --inflight N     concurrent Telegram parts per stream (default 8)
  --max-streams N  concurrent streaming responses (default 256)
  --version        build identity
