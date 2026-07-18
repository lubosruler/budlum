# ARENA2 — P1-R0 AccessGrant Onarım Tasarımı

**Durum:** Tasarım ve kabul kriteri; kod değişikliği değildir.

## Amaç

P1-R0, AccessGrant’ı “varmış gibi görünen” bir yapıdan, sonraki güvenlik işlerinin dayanabileceği açık bir temel haline getirir. Bu aşamada ödeme, şifreleme veya yeni RPC uygulanmaz.

## Önce çözülmesi gereken gerçek

Yeni erişim/market işlemleri zincir üzerinden taşınacaksa P0 transaction protobuf sorunu kapanmalıdır. Mevcut P2P dönüşümü yeni işlem türlerini sessizce Transfer’a çevirebildiğinden, AccessGrant için yeni transaction türü eklemek P0’dan önce güvenli değildir.

Bu yüzden P1-R0 iki ayrı sınır çizer:

1. **Derleme ve veri modeli düzeltmesi:** Hatalı placeholder kaldırılır; bağımsız unit testlerle temizlenir.
2. **Zincire bağlı mutation yolu:** P0 typed-protobuf ve activation tasarımı olmadan uygulanmaz.

## Önerilen principal ve çağıran model

- Grant’in asıl alıcısı her zaman **Address** olur.
- `RoleId`, “bu adres AI verifier olarak kayıtlı mı?” gibi uygunluk kontrolünde kullanılır; tek başına erişim izni değildir.
- `register_asset`, `submit_grant`, `revoke_grant`, `list_asset` ve `purchase` için çağıran Address, imzalı canonical işlemden gelir. RPC parametresindeki yazı adresi kimlik kanıtı değildir.
- Owner kontrolü: `caller == asset.owner`; aksi durumda deterministik red.

## İmza biçimi

İmza alanları `Vec<u8>` değil, exact 64-byte doğrulanmış değer olmalıdır. Her mesaj ayrı etiket taşır:

- `BDLM_STORAGE_COMMITMENT_V1`
- `BDLM_ACCESS_GRANT_V1`
- `BDLM_ACCESS_REVOCATION_V1`
- `BDLM_MARKETPLACE_AUTH_V1`

İmzalanan veri length-prefixed, domain-separated ve en az chain id, asset id, caller/grantee Address, scope, scope expiry, data-scope/version ve authorization nonce içerir. Boş, yanlış uzunlukta veya doğrulanmayan imza fail-closed reddedilir.

## Otomatik satış

Otomatik satış, zincirin owner adına imza üretmesi değildir. Owner önceden şunları bağlayan satış yetkisi verir:

- değişmez veri kapsamı/sürümü,
- fiyat ve protocol fee kuralı,
- izin türü,
- geçerlilik sonu,
- tekil satış/authorization nonce,
- alıcı Address veya açıkça sınırlı alıcı politikası.

Ödeme ve grant oluşumu aynı state transition’da olur; biri başarısızsa hiçbiri uygulanmaz.

## ReadOnce

`ReadOnce`, grant’in yanında ayrı consumption kaydı olmadan sunulmaz. Kayıt grant id + kullanım nonce + block height taşır. Aynı grant ikinci kez tüketilirse reddedilir. Bu yalnız sonraki yetkili erişimi yönetir; geçmişte kopyalanmış veriyi geri almaz.

## P1-R0 kabul kriterleri

1. `if !asset.owner == listing.asset_id` placeholder’ı kaldırılmış ve gerçek caller/owner kontrolü için uygun API tasarımı belgelenmiş.
2. Boş/65-byte/geçersiz imza, zero owner/grantee ve yanlış domain message negatif testleri.
3. Role grant’in tüm role üyelerine erişim vermediğini gösteren Address negatif testi.
4. Auto-grant’ın boş owner imzası oluşturmadığını gösteren test.
5. P0 kapanmadan yeni AccessGrant transaction/RPC mutation yolu eklenmediğini gösteren kapsam testi.
6. fmt + clippy + unit test CI kanıtı.

## Kullanıcıdan sonraki kararlar

1. Otomatik satış yetkisi yalnız tek alıcıya mı bağlanmalı, yoksa süre/fiyat/kapsamla sınırlı açık alıcı modeli de kabul edilsin mi?
2. Data scope ilk sürümde bütün immutable manifest mi, yoksa shard/subset seçimi de mi desteklensin?
3. P1-R0 yalnız derleme+tasarım düzeltmesi olarak mı pushlansın, yoksa P0 transport kapanana kadar hiç kod değişikliği olmasın mı?
