
# Asset Pipeline (AVIF, Sprites, CSS)

## Sources

- Versions: `https://ddragon.leagueoflegends.com/api/versions.json` (latest = index 0)
- Items: `.../cdn/{version}/data/en_US/item.json` + `.../img/item/{id}.png`
- Summoner Spells: `.../cdn/{version}/data/en_US/summoner.json` + `.../img/spell/{name}.png`
- Perks: CommunityDragon JSON + `.../cdn/{version}/data/en_US/runesReforged.json`
- Champions (square): `https://cdn.communitydragon.org/{version}/champion/{id}/square`
- Profile Icons: `.../cdn/{version}/data/en_US/profileicon.json` + `.../img/profileicon/{id}.png`

Concurrent downloads with exponential backoff.

## Paths

- Temp (PNG): `asset-generation/tmp/{items|summoner_spells|perks|champions|profile_icons}`
- Final AVIF: `ruche/public/assets/...`
- Stylesheets: `ruche/style/{items|summoner_spells|perks|champions}.css`
- Sprite AVIF: `/assets/{items|summoner_spells|perks|champions}.avif`
- Logo: `/assets/logo.avif` from `asset-generation/tmp/logo.png`

## Sprited vs Standalone

- **Sprited**: Items, Spells, Perks, Champions (background-image classes).
- **Standalone**: Profile Icons, Logo (`<img src>`).

## Conversion

- Default sizes (px): Items **22×22**, Spells **22×22**, Perks **28×28**, Champions **48×48**, Profile Icons **64×64**
- AVIF encoder: quality **75**, speed **1** (logo quality **100**)

## Layout & CSS

- Grid: `ceil(sqrt(n)) × ceil(sqrt(n))`
- One AVIF sprite per asset type; one CSS class per ID with `background-position`, `width`, `height`.

Class naming:

- Items: `.ii-<itemId>`
- Spells: `.ss-<spellId>`
- Perks: `.pk-<perkId>`
- Champions: `.cn-<championId>`

### Usage

```html
<span class="ii-3031" aria-label="Infinity Edge"></span>
<span class="ss-4" aria-label="Flash"></span>
<i class="pk-8369" aria-label="First Strike"></i>
<div class="cn-266" title="Aatrox"></div>
<img src="/assets/profile_icons/1234.avif" alt="Profile Icon #1234" width="64" height="64">
````

## CLI

```bash
cargo run --bin asset-generation --release
cargo run --bin asset-generation --release -- --help
# Force specific groups
cargo run --bin asset-generation --release -- \
  --items --summoner-spells --perks --champions --profile-icons --logo
```

## Notes

* Re-run after Riot patch changes.
* Regenerate only changed groups via flags.
* Very large new IDs may require adjusting sprite grid sizing.
