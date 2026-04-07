# <p style="text-align: center">qbittorrent_api</p>

An async Rust API client for [qBittorrent](https://github.com/qbittorrent/qBittorrent) v4.1 through v5.0+.

## Scope

This crate intentionally provides a small subset of the qBittorrent WebUI API, covering only the endpoints needed by dependent projects:

- **Authentication** - login with session cookie persistence
- **Torrent listing** - `GET /torrents/info` with filtering, sorting, and pagination
- **Torrent upload** - `POST /torrents/add` with multipart file upload

Other endpoints (pause/resume, delete, categories, tags, trackers, preferences, etc.) are not implemented. Contributions are welcome.

## API field coverage

The `Torrent` response struct and `State` enum are based on the [qBittorrent source code](https://github.com/qbittorrent/qBittorrent/blob/release-5.0.0/src/webui/api/serialize/serialize_torrent.cpp), not the wiki documentation. The [wiki API docs](https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)) are significantly out of date and missing many fields that the API actually returns (e.g. `comment`, `private`, `reannounce`, `popularity`, `infohash_v1`, `infohash_v2`).

Fields present in all versions from v4.1.0 onwards are required. Fields added in later versions (v4.4+, v4.5+, v4.6+, v5.0+) are `Option` for backwards compatibility.

## Releases and changes

Releases and a full changelog are available via [GitHub Releases](https://github.com/RogueOneEcho/qbittorrent_api/releases).

Release versions follow the [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html) specification.

Commit messages follow the [Conventional commit](https://www.conventionalcommits.org/en/v1.0.0/) specification.
