# cosmox-plugin-anime

Base anime plugin for cosmox.

## Description

Handles local metadata tree reconstruction (pre-scrape) for anime media. This plugin processes `OnMetadataRawTreeReady` events and rebuilds the file-based metadata tree into an anime-friendly structure (seasons, episodes).

Does **not** make any network calls. Online scraping plugins (anilist, myanimelist, tmdb, bangumi) depend on this crate for shared data structures and utility functions.
