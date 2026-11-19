# Alacran Tweak Tool – Düzeltme Notları

Kaynak: GitHub release `tools` → `Alacran.Tweak.Tool.bat` (v1.1)
Düzeltilmiş dosya: `Alacran.Tweak.Tool.bat` (bu klasörde)

> Dosya yalnızca metin olarak düzenlendi. Hiçbir aşamada çalıştırılmadı ve test edilmedi.
> Kendi bilgisayarında denemeden önce bir sanal makinede (VM) ya da geri yükleme noktası alarak dene.

Satır numaraları **orijinal** dosyaya göredir.
Devre dışı bırakılan satırlar silinmedi; başında `REM [FIX] <sebep>:` ile yorum satırına çevrildi.
`REM [FIX] ...:` kısmını silersen o satır eski haline döner.

## Çökmeye veya yanlış davranışa yol açan hatalar

| Satır | Sorun | Düzeltme |
|---|---|---|
| 718, 818, 2867 | `for ... do (` blokları hiç kapanmıyordu, menüler "ELSE was unexpected" hatasıyla çöküyordu | Eksik `)` eklendi |
| 810 | Kapanmayan, değişkeni de kullanmayan `for` sarmalayıcısı | Kaldırıldı, içindeki komutlar bir kez çalışıyor |
| 5416 | `IF %a%==1 (` satırında kapanış parantezi yoktu | `)` eklendi |
| 830 / 868 / 882 / 1206 | Power menüsünde yanlış tuşa basınca bütün güç tweak'leri onaysız uygulanıyordu (aynı adlı iki etiket) | Menü etiketi `:insanepowermenu` olarak ayrıldı |
| 4018 | `goto SvcHostMenu` → böyle bir etiket yoktu | `goto svc` |
| Tüm menüler | Boş Enter'a basınca script kapanıyordu, önceki seçim tekrar çalışabiliyordu | `set "input="` sıfırlama ve tırnaklı karşılaştırma (`"%input%"=="1"`) – 26 + 9 giriş, 156 karşılaştırma |
| 288 | 21. seçenek YouTube'u açıp "Misspell" hatasına düşüyordu | `& goto menu` eklendi |
| 379–387 | `%%a`, `%%s`, `%%i` döngü dışında kullanılmıştı, sahte `Enum\%a\...` anahtarı oluşuyordu | Gerçek USB cihazları üzerinde dönen bir döngüye çevrildi |
| 3097 | GPU döngüsü yalnızca `set` komutunu çalıştırıyordu, gövde döngü dışında kalıyordu | `do (` olarak düzeltildi |
| 2816–2817, 3099, 3102 | "Deleting/Removing" adımları `reg add` kullanıyor, boş değer yaratıyordu | `reg delete` |
| 1403 | `%ProcessorManufacturer%` hiç atanmıyordu, AMD dalı hiç çalışmıyordu | İşlemci üreticisi PowerShell ile okunuyor |
| 65, 792–795, 810, 818, 2815, 2821, 2867, 3097 | `wmic` Windows 11 24H2'de kaldırıldı | `Get-CimInstance` (PowerShell) |
| 215–216 | Geri yükleme noktası başarısız olsa bile "başarıyla oluşturuldu" diyordu | Sonuç kontrol ediliyor, uygun mesaj gösteriliyor |

## Yazım hataları

| Satır | Önce | Sonra |
|---|---|---|
| 399 | `Disablingdynamictick` | `disabledynamictick` |
| 1391, 1399, 3152–3163 | `EnablingVirtualizationBasedSecurity`, `EnablingCfg`, `EnablingUlps`, … (8 değer) | `Enable…` |
| 3581, 3584 | `Disabling_OverlayDSQualityEnhancement`, `DisablingPFonDP` | `Disable_…`, `DisablePFonDP` |
| 3578 | `cho` | `echo` |
| 4787 | `useplatformclockJ` | `useplatformclock` |
| 5535 | `rd s q` | `rd /s /q` |
| 5762 | `cdedit` | `bcdedit` |
| 912 | Yarıda kesik satır, `REG_SZ` | `REG_DWORD … /f` |
| 1153 | `scheme_curre` | `scheme_current` |
| 4169 | `EnableSuperfetch … /t REG_D` | `/t REG_DWORD /d "0" /f` |
| 5169 | `"CPCONCUR` (yarıda kesik, değeri bilinmiyor) | Satır kaldırıldı |
| 1182, 2830, 3110, 3346, 3618 | `GpuEnergyDr` (yanlış servis adı, doğru adlı satırın kopyası) | Satır kaldırıldı |
| 8 | Kapanmayan tırnak | Düzeltildi |
| 60 | Komut olarak çalışan `░%p%` satırı | Kaldırıldı |

## Güvenlik / veri kaybı – devre dışı bırakıldı (REM)

| Satır | Komut | Sebep |
|---|---|---|
| 1268 | `rd "%USERPROFILE%\OneDrive" /q /s` | Kullanıcının OneDrive dosyalarını **kalıcı olarak siliyordu** |
| 1333 | Tüm Image File Execution Options anahtarlarını silme | Başka programların ayarlarını ve aracın kendi csrss ayarını da siliyordu |
| 2500 | `wevtutil cl` (tüm olay günlüklerini silme) | Sorun gidermeyi imkânsız hale getiriyordu |
| 4796 | `bcdedit integrityservices disable` | Kod bütünlüğü denetimini zayıflatıyordu |
| 4800 | `bcdedit disableelamdrivers yes` | Antivirüsün erken başlatma korumasını kapatıyordu |
| 4803 | `bcdedit pae ForceDisable` | 64-bit Windows'ta PAE zorunlu |
| 5468 | `bcdedit increaseuserva 8000` | Yalnızca 32-bit Windows'ta anlamlı |
| 5504 | `EnableLUA = 0` | 1080p sorusuna "Hayır" demek UAC'yi kapatıyordu (kopyala-yapıştır hatası). Artık bir şey değiştirmiyor |
| 5503 | `C:\Windows\AtlasModules\nsudo …` | Yalnızca AtlasOS'ta bulunan bir dosya. Yerine ölçeklendirmeyi %100 yapan registry ayarı kondu |

## Ağ ayarları – devre dışı bırakıldı (REM)

| Satır | Ayar | Sebep |
|---|---|---|
| 711, 712, 780, 781 | `TcpWindowSize` / `GlobalMaxTcpWindowSize = 8760` | TCP penceresini 8,7 KB'a düşürüp hızı kısıtlıyor |
| 715, 784 | `SackOpts = 0` | Paket kaybında performansı düşürüyor |
| 778 | `MaxSockAddrLength = 16` | IPv6 adresleri 28 bayt, IPv6 bağlantıları bozulabilir |
| 689, 691, 695, 696, 803 | `netsh … chimney / dca / netdma / congestionprovider` | Windows 10/11'de desteklenmiyor, hata veriyor |

## Davranışı değişen diğer yerler

- **3855 – Chris Titus WinUtil:** Script'i internetten indirip çalıştırmadan önce artık açıklama gösterip `[Y/N]` onayı istiyor.
- **Başlangıç:** `%temp%` içindeki VBS dosyası ve `System32\adminrightstest` klasörü artık işlem bitince siliniyor.
- **483–504:** Hemen üstündeki görev zamanlayıcı bloğunun birebir kopyasıydı, kaldırıldı.

## Dokunulmayanlar (bilinçli tercih)

Bunlar aracın asıl amacı olan "performans için güvenlikten taviz" tweak'leri. Hata olmadıkları için bırakıldılar, ancak riskli olduklarını bilmekte fayda var:

- Spectre/Meltdown, CFG, ASLR ve sistem exploit korumalarının kapatılması (1332, 5480)
- `hypervisorlaunchtype off`: WSL2, Hyper-V, Windows Sandbox ve Android alt sistemi çalışmaz
- `tpmbootentropy ForceDisable`, `nx optout`
- Windows Update'in kapatılması, `$Recycle.Bin`'in boşaltılması, Kamera/Posta gibi uygulamaların kaldırılması

## Hâlâ açık olanlar

- `%color1%`, `%colorA%`, `%colorC%`, `%colorD%` hiçbir yerde tanımlı değil, bu yüzden o renkler görünmüyor. Hangi renkleri istediğin bilinmediği için dokunulmadı.
- 5169. satırdaki `CPCONCUR…` ayarının asıl değeri dosyadan çıkarılamıyor, bu yüzden satır kaldırıldı. 4169'daki `EnableSuperfetch` için `0` değeri tahmin edildi (hemen üstündeki `EnablePrefetcher = 0` ile uyumlu). Elinde tam sürüm varsa bu ikisini oradan kontrol et.
