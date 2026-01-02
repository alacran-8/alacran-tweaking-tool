# Alacran Tweaking Tool

**Alacran Tweaking Tool**, Windows için geliştirilmiş bir **Batch (.bat)** tabanlı sistem ve oyun optimizasyon aracıdır.

Amaçları:
- Oyun performansını artırmak
- Sistem tepkisini hızlandırmak
- Gecikmeyi (latency / input lag) azaltmak
- Windows’u daha sade ve verimli hale getirmek

Bu araçta **zararlı yazılım, arka plan servisi veya gizli işlem yoktur**.

---

## ⚙️ Bu araç ne yapar?

Bu batch dosyası, Windows üzerinde aşağıdaki işlemleri uygular:

- Güç planı (power plan) optimizasyonları
- Gereksiz Windows servislerinin devre dışı bırakılması
- Kayıt defteri (registry) üzerinden performans ayarları
- Oyun odaklı sistem tweak’leri
- Temel yapılandırma ve temizlik işlemleri

Tüm komutlar **standart Windows komutlarıdır** ve dosya tamamen **okunabilir / şeffaftır**.

---

## 🛡️ VirusTotal & False Positive Açıklaması

Bazı antivirüs motorları bu aracı **şüpheli (suspicious)** olarak işaretleyebilir.  
Bu durum bir **false positive (yanlış pozitif)** sonucudur.

### Bunun sebepleri:

- Araç Windows kayıt defterini değiştirir
- Yönetici (administrator) yetkisi gerektirir
- Batch dosyaları kötü amaçlı yazılımlar tarafından da kullanılabildiği için
  heuristik taramalarda alarm üretebilir
- Dosya imzalı değildir (code signing yoktur)
- Yeni veya az bilinen bir araç olması (düşük reputation)

Bu script içerisinde **kötü amaçlı hiçbir kod bulunmamaktadır**.

İsteyen herkes:
- `.bat` dosyasını Not Defteri ile açabilir
- Tüm komutları satır satır inceleyebilir
- Ne çalıştığını net şekilde görebilir

---

## ▶️ Nasıl kullanılır?

1. `.bat` dosyasını **Releases** bölümünden indirin
2. Dosyaya sağ tıklayın → **Yönetici olarak çalıştır**
3. Ekrandaki talimatları takip edin
4. İşlem bittikten sonra sistemi yeniden başlatın

---

## ⚠️ Sorumluluk Reddi (Disclaimer)

Bu araç sistem ayarlarını değiştirir.

- Kullanım tamamen sizin sorumluluğunuzdadır
- Çalıştırmadan önce sistem geri yükleme noktası oluşturmanız önerilir
- Yanlış veya bilinçsiz kullanım sonucu oluşabilecek sorunlardan geliştirici sorumlu değildir

---

## 📜 Lisans

MIT License
