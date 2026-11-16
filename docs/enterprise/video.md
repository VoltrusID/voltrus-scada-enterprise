# RTSP video cameras

IP cameras (Hikvision, Dahua, Axis or any RTSP source) registered on the
server: a live view in the browser, snapshots on demand, snapshots taken
automatically when an alarm is raised, and per-camera retention. Licence:
Enterprise (`rtsp_video`). Code: `enterprise/src/video/`.

## How it works

The server, not the browser, talks to the camera. It runs **ffmpeg**
(RTSP over TCP, Basic or Digest authentication, H.264, H.265 or MJPEG):

* **Snapshot**: one frame as JPEG, saved under `snapshots/` next to the
  database file, with a row in `camera_snapshots` (time, size, the alarm that
  triggered it).
* **Live view**: `GET /api/v1/cameras/<id>/live` transcodes the stream to MJPEG
  (`multipart/x-mixed-replace`, 5 frames/s, at most 1280 px wide) that an
  `<img>` plays without plug-ins. The camera's address and password never
  reach the browser; the browser needs a Voltrus session. At most 8 live views
  are open at once over all cameras (each is an ffmpeg process that ends when
  the viewer closes it).
* **On alarm**: every 2 s the active node looks for alarms raised since the
  last look and takes one snapshot from every enabled camera with **Snapshot on
  Alarm**, at most one per camera per 30 s, labelled `alarm <id>: <tag>
  (<message>)`.
* **Retention**: once an hour, snapshots older than the camera's
  `retention_hours` are deleted (files and rows). A manual capture also prunes
  that camera.

It is a relay with transcoding, not a pass-through: the server decodes and
re-encodes, so plan about one CPU core for every two or three 1080p live
views. It is not a video recorder (no continuous recording or playback).

## Set up

1. Install ffmpeg on the server (`apt install ffmpeg`, `dnf install ffmpeg`,
   Windows: put `ffmpeg.exe` on the PATH). Another location:
   `VOLTRUS_FFMPEG=/opt/ffmpeg/bin/ffmpeg` in the service environment.
2. Licence: **More actions (⋯) › Data Manager › Admin › License** shows
   Enterprise.
3. Find the camera's RTSP URL (camera manual or ONVIF device manager), for
   example `rtsp://user:pass@192.168.1.64:554/Streaming/Channels/102` for a
   Hikvision sub-stream. Prefer the sub-stream: it is cheaper to transcode.
   Give the camera its own read-only account.
4. As an admin: **Data Manager › Data › Cameras › + Add Camera**: Name, RTSP
   URL, Snapshot on Alarm, Retention (hours). **Add Camera**. API:
   `POST /api/v1/cameras` with `{"name","url","enabled","snapshot_on_alarm","retention_hours"}`.
5. In the camera's row: **Capture** takes a snapshot, **Snapshots (n)** lists
   them with **View** (opens the JPEG), **Live** shows the live view.

## Check it works

1. **Capture** returns quickly and **Snapshots** gains a row marked Manual;
   **View** shows the picture (`GET /api/v1/cameras/<id>/snapshots/<sid>/file`).
2. **Live** shows moving video within 2 to 3 seconds.
3. Raise an alarm (for a test, lower a tag's critical limit below its value):
   within a few seconds the camera has a snapshot whose source reads
   `alarm <id>: …`, and the log has `camera <name>: snapshot for alarm …`.
4. `GET /api/v1/cameras/<id>/stream` shows the URL with the password as `***`.

Verified on 2026-11-16 against mediamtx 1.15.3 serving an H.264 test pattern
behind Basic/Digest authentication: snapshot (640x360 JPEG), live view (31
frames in 8 s, ffmpeg gone after the viewer closed), a wrong password reported
as `401 Unauthorized` with the password masked, an alarm snapshot taken 2 s
after a critical alarm, no snapshot for a camera without Snapshot on Alarm,
401 for the live view without a session.

## Troubleshooting

| Message | Cause and fix |
| --- | --- |
| `ffmpeg was not found` | Install ffmpeg or set `VOLTRUS_FFMPEG`. |
| `… 401 Unauthorized (authorization failed)` | Wrong camera user or password in the URL. Special characters in the password must be URL-encoded (`@` as `%40`). |
| `camera … gave no frame within 20s` | Wrong path, camera unreachable from the server, or a firewall blocks TCP 554. Test on the server with `ffmpeg -rtsp_transport tcp -i '<url>' -frames:v 1 test.jpg`. |
| `8 live views are open already` | Close other live views; each viewer holds one. |
| `Snapshot file is missing on this node` | After a hot standby failover: snapshot files are not replicated (their rows are). |
| No alarm snapshots | Camera disabled or Snapshot on Alarm off; or the node is the standby. Snapshots of one camera are at least 30 s apart. |

## Not available

* **Camera tiles on HMI screens and dashboards.** The core's screens have no
  slot for add-on widgets, so cameras are viewed in **Data Manager › Cameras**
  only. Needs a widget hook in the open core.
* **Native console**: no camera screen.
* Roles: the camera list and controls are admin only. The live view and the
  snapshot image are also open to the Demo (operator) role and integration API
  keys by URL; Viewer and Operator-role accounts get `403`.

## Fixed in the audit of 2026-11-16

Before: snapshots used a built-in RTSP client that asked for UDP transport but
read the TCP control connection (so it hung or saved non-JPEG bytes), knew only
Basic authentication and saved H.264 data as `.jpg`; there was no live view
(the stream endpoint returned the camera URL with its password), no endpoint
for the snapshot image, snapshot on alarm was never triggered, retention never
ran, and snapshots went to `./snapshots` in the working directory.
