# Third-party notices

`rhobuilder` is AGPL-3.0-only.

## DeepSeek Harness

DeepSeek Harness (DSH) is MIT-licensed. It is a runtime dependency only; no DSH source is
vendored. The plugin and launcher patterns are adapted from the azoth DSH integration
(`PatrickMockridge/azoth`, branch `dsh-integration`, commit `8b7810ee`), which is
AGPL-3.0-or-later. The adapted code remains under rhobuilder's AGPL-3.0-only licence, as
permitted by the "or later" option.

## Sibling rhovision repositories

The registry references r-cash, r-sure, r-gen, r-share, r-print and r-bridge by pinned
commit. Templates and gates that copy material from a sibling carry a per-file origin
marker naming the source repo, commit and licence. Sibling repositories are
AGPL-3.0-or-later; the compatibility basis is recorded per vendored file.

No sibling repository is modified by rhobuilder.
