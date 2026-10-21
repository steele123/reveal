# Discord artwork sources

Rendered with headless Microsoft Edge (sizes in CSS pixels; the banner renders at 2x):

```powershell
$edge = "${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe"
# Server icon, 512x512
& $edge --headless=new --window-size=512,512 --screenshot=icon.png file:///.../icon.html
# Banner (960x540) and invite background (1920x1080): render once at 2x, then downscale for the banner
& $edge --headless=new --force-device-scale-factor=2 --window-size=960,540 --screenshot=banner-2x.png file:///.../banner.html
# Emojis, 128x128 with transparent background (pass the image after #)
& $edge --headless=new --window-size=128,128 --default-background-color=00000000 --screenshot=gold.png "file:///.../emoji.html#gold.svg"
```

- `banner.html` expects `icon.png` (from `icon.html`) and `app-clean.png` next to it.
- `app-clean.png` is the app's Champ Select preview (`pnpm dev`, `/?preview=champ-select`) with the browser-only "Preview" badge covered.
- Rank crest SVGs are in `public/ranks/`. The `reveal` emoji is the icon with rounded corners.
- Edge's `--headless` launcher returns early; use `Start-Process -Wait` and wait for the file.
