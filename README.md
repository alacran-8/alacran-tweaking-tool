# Alacran Tweak Tool 2.0

**Türkçe** · [English](#english)

Windows 10 (21H2 ve üstü) ve Windows 11 için ücretsiz, tek dosyalık bir ayar (tweak) uygulaması. Eski `Alacran.Tweak.Tool.bat` betiğinin yerini alır.

- **Tek dosya:** `AlacranTweak.exe`. Kurulum yok, internete bağlanmaz.
- **Geri alınabilir:** Her uygulamadan önce değiştirilecek değerler yedeklenir. "Geri yükle" sayfasından tek tek ya da hepsi birden geri alınır. İlk uygulamadan önce bir sistem geri yükleme noktası oluşturulur.
- **Dürüst açıklamalar:** Her ayarın ne yaptığı ve ne zaman fark etmeyeceği yazar. Riskli ayarlar **DİKKAT**, geri alınamayanlar (temizlik, uygulama kaldırma) ayrıca onay ister.
- **Tek tık optimizasyon:** Büyük düğme yalnızca güvenli ayarları uygular.

## Gereksinimler

- Windows 10 21H2+ veya Windows 11, 64 bit
- Yönetici izni (açılışta UAC sorar)
- Microsoft WebView2 Runtime (Windows 11'de hazır gelir; yoksa uygulama indirme sayfasını açmayı önerir)
- "Sistem Koruması" sistem sürücüsünde açık olmalı (geri yükleme noktası için)

## İndirme

[Releases](https://github.com/AIacran/alacran-tweaking-tool/releases) sayfasından `AlacranTweak.exe`'yi indir. İndirdiğin dosyanın SHA-256 değerini sürüm notundakiyle karşılaştırabilirsin:

```powershell
Get-FileHash .\AlacranTweak.exe -Algorithm SHA256
```

Dosya imzalı değildir; Windows SmartScreen uyarı gösterebilir.

## Deneme modu

`AlacranTweak.exe --dry-run` hiçbir şeyi değiştirmez; yalnızca ne yapacağını günlüğe yazar.

## Eski sürüm (1.1, .bat)

Eski Batch sürümü [`legacy/`](legacy) klasöründe ve [Releases](https://github.com/AIacran/alacran-tweaking-tool/releases) sayfasında duruyor. Hataları düzeltilmiş kopyası ve düzeltme notları: [`legacy/fixed/`](legacy/fixed).

## Yedekler ve günlükler

- Yedekler: `C:\ProgramData\AlacranTweak\backups`
- Günlük: `C:\ProgramData\AlacranTweak\apply.log`

---

## English

A free, single-file tweak app for Windows 10 (21H2+) and Windows 11, replacing the old `Alacran.Tweak.Tool.bat`.

- **Single file:** `AlacranTweak.exe`. No installer, no network access.
- **Revertible:** values are backed up before every apply and can be restored one by one or all at once from the Restore page. A system restore point is created before the first apply.
- **Honest texts:** each tweak says what it does and when it makes no difference. Risky tweaks are marked **CAUTION**; one-way actions (cleanup, app removal) ask for confirmation.
- **One-click optimise:** the big button applies safe tweaks only.

Requirements: Windows 10 21H2+ or Windows 11 x64, administrator rights, WebView2 Runtime, System Protection on for the system drive.

Download `AlacranTweak.exe` from [Releases](https://github.com/AIacran/alacran-tweaking-tool/releases) and compare its SHA-256 with the release notes. The file is not code-signed. `--dry-run` changes nothing and only logs what it would do.

## Build (developers)

Needs Rust, Node.js and Visual Studio 2022 Build Tools.

```bat
cd alacran-tweak-app
npm install
scripts\test.cmd
npx vitest run
scripts\build.cmd
```

Output: `C:\at-target\release\AlacranTweak.exe`. Apply/revert is tested only in a throwaway VM: see [docs/vm-testing.md](docs/vm-testing.md).

The old Batch version (1.1) is kept in [`legacy/`](legacy).

## Lisans / License

MIT, see [LICENSE](LICENSE). Bundled libraries: Tauri (MIT / Apache-2.0), threejs-components (ISC), three.js r180 (MIT).
