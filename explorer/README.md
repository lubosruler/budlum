# Budlum Devnet + Block Explorer

Yerel geliştirme için tek-komut multi-node devnet ve zincir durumunu Rust kodu
okumadan izlemeyi sağlayan salt-okunur block explorer.

## 1) Devnet — `devnet.sh` (tek komut)

Kök dizindeki `devnet.sh`, mevcut `docker-compose.yml` (4 node, PoW difficulty=0,
RPC + metrics + Prometheus) üzerinde bir sarmalayıcıdır.

```bash
./devnet.sh up        # derle + başlat + sağlık kontrolü (varsayılan komut)
./devnet.sh status    # yükseklik, chain id, peer sayısı, validator/domain
./devnet.sh health    # liveness: listening + peer + blok üretimi (2 ölçüm)
./devnet.sh logs 2    # node2 logları (N=1..4)
./devnet.sh ps        # container durumları
./devnet.sh rpc bud_blockNumber            # ham RPC (node1)
./devnet.sh rpc bud_getBalance '["<addr>"]'
./devnet.sh down      # durdur (volume'lar kalır)
./devnet.sh reset     # durdur + volume sil (temiz zincir)
```

Ön koşullar: `docker` + `docker compose` (v2 plugin). Ortam değişkenleri:
`RPC_URL` (varsayılan `http://127.0.0.1:8545`), `METRICS`, `COMPOSE` (proje adı),
`WAIT_SECS` (sağlık kontrolü üst sınırı).

`scripts/devnet-multinode-smoke.sh` CI'da aynı uçlarla liveness/güvenlik
iddialarını mühürler; `devnet.sh` operatör/kullanıcı odaklı günlük kullanım içindir.

## 2) Block Explorer — `explorer/index.html`

Tek dosya, sıfır bağımlılık (inline CSS/JS), salt-okunur paneldir — hiçbir
transaction/imza göndermez.

```bash
./devnet.sh up                 # 1) ağı başlat
# 2) explorer/index.html'i tarayıcıda aç
```

Panel şunları gösterir (otomatik 5 sn yenileme):
- **Durum kartları:** yükseklik (`bud_blockNumber`), chain id (`bud_chainId`),
  peer sayısı (`bud_netPeerCount`), listening (`bud_netListening`), sağlık (`bud_health`).
- **Son bloklar:** `bud_getBlockByNumber`; tıklayınca blok detayı + transaction'lar
  (hash, tip, from/to, amount, fee, nonce).
- **Validator'lar** (`bud_getValidatorSet`) ve **konsensüs domain'leri** (`bud_getConsensusDomains`).
- **Hesap sorgula:** adres → `bud_getBalance` + `bud_getNonce`.
- **Ham RPC:** herhangi bir `bud_*` metodu (params JSON ile).

### CORS notu
Tarayıcıdan node RPC'sine `fetch` yapılır. Node, sayfanın origin'ine izin
vermelidir. Sorun yaşarsanız: node'u CORS açık çalıştırın, sayfayı node ile aynı
origin'den sunun veya yerel bir CORS proxy kullanın. Bağlantı kopukluğu panelde
net bir hata kutusuyla gösterilir.

## Kapsam / sınırlamalar
- Explorer salt-okunurdur (yazma yok); imza/transaction göndermez.
- `devnet.sh` PoW difficulty=0 devnet'i içindir (üretim/mainnet değil).
- Docker imajı `Dockerfile` ile çok-aşamalı derlenir (`cargo build --release`).
