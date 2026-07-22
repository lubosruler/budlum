// Copyright (c) 2026 Budlum. All rights reserved.
//! BnsHooks — budlumCore tarafından tanımlanan, B.U.D. tarafından uygulanan trait.
//!
//! Konsolidasyon Faz 4: BNS (B.U.D. Name Service) modülü budlumCore'dan B.U.D. crate'ine
//! taşındı. budlumCore, BNS işlevselliğine `BnsHooks` trait üzerinden erişir.
//!
//! Tasarım ilkesi (K7 / dependency inversion): B.U.D. → budlumCore (tek yön).
//! budlumCore BNS somut tipine (`BnsRegistry`, `BnsResolved`, `NameRecord`, `BnsError`)
//! bağımlı olmaz; yalnızca bu trait'e ve `BnsResolved`'in **opak bir projection**'ına
//! bağımlıdır. Bu sayede budlumCore B.U.D. opsiyonel eklenti olsa da derlenir ve
//! çalışır (extension = None ise tüm BNS fonksiyonları "bilinmiyor" döner).
//!
//! Test'ler ve diğer modüller (gateway/passport, vb.) trait üzerinden erişir.

use crate::core::address::Address;
use serde::{Deserialize, Serialize};

/// BNS'in dış dünyaya gösterdiği **opak projection**. B.U.D. tarafında bu tip
/// somut `BnsResolved` tipine indirgenir. budlumCore yalnızca bu projection'ı bilir
/// — `NameRecord`, `BnsRegistry`, `BnsError` gibi iç detayları bilmez.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BnsResolvedView {
    pub name: String,
    pub owner: Address,
    pub address: Option<Address>,
    pub storage_root: Option<[u8; 32]>,
    pub storage_domain_id: Option<u32>,
    pub content_id: Option<[u8; 32]>,
    pub is_expired: bool,
}

/// BNS eklenti trait'i. B.U.D. tarafında uygulanır (`BnsRegistry` somut implementasyonu).
///
/// `Send + Sync` gerekir çünkü Blockchain struct'ı multi-threaded runtime'da
/// (tokio task'ları) bu trait object'leri paylaşır.
///
/// **Fail-closed:** B.U.D. eklentisi yoksa budlumCore default `()` (veya
/// `None`) implementasyonunu kullanır — tüm BNS çağrıları "yok" döner, panik yapmaz.
pub trait BnsHooks: Send + Sync {
    /// Tam isim çözümlemesi. Pasif eklenti (`None`) için `None` döner.
    fn bns_resolve_full(&self, name: &str, current_epoch: u64) -> Option<BnsResolvedView>;

    /// İsim kayıt et (mutating). B.U.D. tarafında `BnsRegistry::register` ile eşlenir.
    ///
    /// **Hata kodları:**
    /// - `Ok(())` — başarı
    /// - `Err(BnsHookError::InvalidName)` — geçersiz isim (uzunluk, karakter)
    /// - `Err(BnsHookError::NameTaken)` — ad zaten alınmış (grace period dahil)
    /// - `Err(BnsHookError::NotOwner)` — sahiplik kontrolü başarısız
    /// - `Err(BnsHookError::Expired)` — kayıt süresi dolmuş
    /// - `Err(BnsHookError::ExtensionDisabled)` — B.U.D. eklentisi takılı değil
    fn bns_register(
        &self,
        name: &str,
        owner: Address,
        current_epoch: u64,
        duration: u64,
    ) -> Result<(), BnsHookError>;

    /// İsim yenileme (renew).
    fn bns_renew(
        &self,
        name: &str,
        caller: &Address,
        current_epoch: u64,
        duration: u64,
    ) -> Result<(), BnsHookError>;

    /// Sahiplik transferi.
    fn bns_transfer(
        &self,
        name: &str,
        caller: &Address,
        new_owner: Address,
        current_epoch: u64,
    ) -> Result<(), BnsHookError>;

    /// Subdomain kayıt.
    fn bns_register_subdomain(
        &self,
        parent_name: &str,
        sub_label: &str,
        owner: Address,
        caller: &Address,
    ) -> Result<(), BnsHookError>;

    /// Subdomain çözümleme.
    fn bns_resolve_subdomain(
        &self,
        parent_name: &str,
        sub_label: &str,
        current_epoch: u64,
    ) -> Option<Address>;

    /// İçerik (ContentId) ata.
    fn bns_set_content(
        &self,
        name: &str,
        owner: &Address,
        content_id: [u8; 32],
    ) -> Result<(), BnsHookError>;

    /// İçerik çözümleme.
    fn bns_resolve_content(&self, name: &str, current_epoch: u64) -> Option<[u8; 32]>;

    /// Basit çözümleme (sadece address).
    fn bns_resolve(&self, name: &str, current_epoch: u64) -> Option<Address>;

    /// Storage root ata.
    fn bns_set_storage(
        &self,
        name: &str,
        caller: &Address,
        storage_root: [u8; 32],
        storage_domain_id: u32,
        current_epoch: u64,
    ) -> Result<(), BnsHookError>;
}

/// BNS hook hata kodu. Somut `BnsError` (B.U.D. tarafında) bu varyanta indirgenir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BnsHookError {
    InvalidName,
    NameTaken,
    NotOwner,
    Expired,
    /// B.U.D. eklentisi yüklü değil. budlumCore default impl'i tüm metodlar için
    /// bu hatayı döner. Kullanıcı bu hatayı görürse: budlum node B.U.D.
    /// eklentisi olmadan çalışıyor; BNS işlemleri için `cargo install` ile
    /// B.U.D. eklentisinin yüklenmesi gerek.
    ExtensionDisabled,
}

impl std::fmt::Display for BnsHookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BnsHookError::InvalidName => write!(f, "BNS: name too short or long"),
            BnsHookError::NameTaken => write!(f, "BNS: name already taken"),
            BnsHookError::NotOwner => write!(f, "BNS: not the owner"),
            BnsHookError::Expired => write!(f, "BNS: registration expired"),
            BnsHookError::ExtensionDisabled => {
                write!(f, "BNS: B.U.D. extension not loaded on this node")
            }
        }
    }
}

impl std::error::Error for BnsHookError {}

/// Default (no-op) implementasyon: B.U.D. eklentisi yoksa kullanılır. Tüm
/// çözümleme metodları `None` döner; mutating metodlar `ExtensionDisabled` hatası.
///
/// **Güvenlik:** Ağ pasif modda çalışır — node başlatılır, B.U.D. opsiyonel
/// eklenti olarak sonradan takılabilir. Bu sayede ağ BNS olmadan da canlanabilir.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullBnsHooks;

impl BnsHooks for NullBnsHooks {
    fn bns_resolve_full(&self, _name: &str, _current_epoch: u64) -> Option<BnsResolvedView> {
        None
    }
    fn bns_register(
        &self,
        _name: &str,
        _owner: Address,
        _current_epoch: u64,
        _duration: u64,
    ) -> Result<(), BnsHookError> {
        Err(BnsHookError::ExtensionDisabled)
    }
    fn bns_renew(
        &self,
        _name: &str,
        _caller: &Address,
        _current_epoch: u64,
        _duration: u64,
    ) -> Result<(), BnsHookError> {
        Err(BnsHookError::ExtensionDisabled)
    }
    fn bns_transfer(
        &self,
        _name: &str,
        _caller: &Address,
        _new_owner: Address,
        _current_epoch: u64,
    ) -> Result<(), BnsHookError> {
        Err(BnsHookError::ExtensionDisabled)
    }
    fn bns_register_subdomain(
        &self,
        _parent_name: &str,
        _sub_label: &str,
        _owner: Address,
        _caller: &Address,
    ) -> Result<(), BnsHookError> {
        Err(BnsHookError::ExtensionDisabled)
    }
    fn bns_resolve_subdomain(
        &self,
        _parent_name: &str,
        _sub_label: &str,
        _current_epoch: u64,
    ) -> Option<Address> {
        None
    }
    fn bns_set_content(
        &self,
        _name: &str,
        _owner: &Address,
        _content_id: [u8; 32],
    ) -> Result<(), BnsHookError> {
        Err(BnsHookError::ExtensionDisabled)
    }
    fn bns_resolve_content(&self, _name: &str, _current_epoch: u64) -> Option<[u8; 32]> {
        None
    }
    fn bns_resolve(&self, _name: &str, _current_epoch: u64) -> Option<Address> {
        None
    }
    fn bns_set_storage(
        &self,
        _name: &str,
        _caller: &Address,
        _storage_root: [u8; 32],
        _storage_domain_id: u32,
        _current_epoch: u64,
    ) -> Result<(), BnsHookError> {
        Err(BnsHookError::ExtensionDisabled)
    }
}

// ============================================================================
// BnsHooks impl for BnsRegistry (somut tip, budlum-core içinde yaşar)
//
// Tasarım gereği: BnsRegistry somut tipi budlum-core'da kalır (K7 dependency
// inversion). Bu impl, BnsRegistry'nin somut metodlarını BnsHooks trait
// çağrılarına bağlar. B.U.D. opsiyonel eklenti kendi BnsHooks impl'ini
// sağlayabilir (örn. zincirler arası BNS, farklı depolama), ama temel
// implementasyon budlum-core'da — budlumCore B.U.D.'ye bağımlı DEĞİL.
//
// BnsError → BnsHookError dönüşümü: 1:1 eşleme (InvalidName/NameTaken/
// NotOwner/Expired varyantları).
// ============================================================================

impl crate::bns::BnsRegistry {
    /// BnsRegistry'yi BnsHooks trait object'ine wrap'ler. Blockchain bunu
    /// kendi `extensions: ExtensionBundle.bns` alanına atar; default olarak
    /// B.U.D. eklentisi yoksa budlum-core'un kendi BnsRegistry'sini kullanır.
    pub fn as_bns_hooks(arc_self: std::sync::Arc<Self>) -> std::sync::Arc<dyn BnsHooks> {
        arc_self
    }
}

impl BnsHooks for crate::bns::BnsRegistry {
    fn bns_resolve_full(&self, name: &str, current_epoch: u64) -> Option<BnsResolvedView> {
        self.resolve_full(name, current_epoch).map(|r| BnsResolvedView {
            name: r.name,
            owner: r.owner,
            address: r.address,
            storage_root: r.storage_root,
            storage_domain_id: r.storage_domain_id,
            content_id: r.content_id.map(|c| c.0),
            is_expired: r.is_expired,
        })
    }

    fn bns_register(
        &self,
        name: &str,
        owner: Address,
        current_epoch: u64,
        duration: u64,
    ) -> Result<(), BnsHookError> {
        // BnsRegistry::register &mut self ister; BnsHooks'ta &self var. İç mutable
        // copy + işlem gerek. Bu **kritik kısıt**: trait inversion her zaman
        // mutating işlemlerde sorun çıkarır. Çözüm: Interior mutability
        // (RefCell) veya copy-on-write. B.U.D. opsiyonel eklenti kendi
        // implementasyonunda bunu çözer; burada BnsRegistry &self olduğu
        // için sadece salt-okunur metodlar uygulanabilir.
        //
        // Güvenlik notu: register gibi mutating işlemler trait üzerinden
        // çağrılamaz; call site'lar doğrudan `state.bns_registry.register(...)`
        // kullanmaya devam eder. Trait sadece **okuma** + **opak projection**
        // için tasarlandı.
        let _ = (name, owner, current_epoch, duration);
        Err(BnsHookError::ExtensionDisabled)
    }

    fn bns_renew(
        &self,
        _name: &str,
        _caller: &Address,
        _current_epoch: u64,
        _duration: u64,
    ) -> Result<(), BnsHookError> {
        Err(BnsHookError::ExtensionDisabled)
    }

    fn bns_transfer(
        &self,
        _name: &str,
        _caller: &Address,
        _new_owner: Address,
        _current_epoch: u64,
    ) -> Result<(), BnsHookError> {
        Err(BnsHookError::ExtensionDisabled)
    }

    fn bns_register_subdomain(
        &self,
        _parent_name: &str,
        _sub_label: &str,
        _owner: Address,
        _caller: &Address,
    ) -> Result<(), BnsHookError> {
        Err(BnsHookError::ExtensionDisabled)
    }

    fn bns_resolve_subdomain(
        &self,
        parent_name: &str,
        sub_label: &str,
        current_epoch: u64,
    ) -> Option<Address> {
        crate::bns::BnsRegistry::resolve_subdomain(self, parent_name, sub_label, current_epoch)
    }

    fn bns_set_content(
        &self,
        _name: &str,
        _owner: &Address,
        _content_id: [u8; 32],
    ) -> Result<(), BnsHookError> {
        Err(BnsHookError::ExtensionDisabled)
    }

    fn bns_resolve_content(&self, name: &str, current_epoch: u64) -> Option<[u8; 32]> {
        crate::bns::BnsRegistry::resolve_content(self, name, current_epoch).map(|c| c.0)
    }

    fn bns_resolve(&self, name: &str, current_epoch: u64) -> Option<Address> {
        crate::bns::BnsRegistry::resolve(self, name, current_epoch)
    }

    fn bns_set_storage(
        &self,
        _name: &str,
        _caller: &Address,
        _storage_root: [u8; 32],
        _storage_domain_id: u32,
        _current_epoch: u64,
    ) -> Result<(), BnsHookError> {
        Err(BnsHookError::ExtensionDisabled)
    }
}
