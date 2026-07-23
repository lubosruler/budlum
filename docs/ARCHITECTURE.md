# Budlum Mimari Atlası

> **Durum:** Kod haritası ve hedef mimariyi birlikte gösterir. Diyagramlardaki
> `feature-gated`, `planlı` ve `mainnet blocker` etiketleri tamamlanmış üretim
> özelliği iddiası değildir.

## 1. Genel sistem mimarisi

```mermaid
flowchart TB
  Client[Wallet / dApp / Operator] --> RPC[JSON-RPC + admission]
  RPC --> MP[Mempool]
  MP --> Chain[ChainActor / Blockchain]
  Chain --> Exec[Transaction Executor]
  Chain --> Consensus[PoW / PoS / PoA / BFT finality]
  Exec --> State[(AccountState)]
  Consensus --> State
  State --> Store[(Durable storage + snapshots)]
  Chain --> Net[P2P / Gossip]
  Exec --> ZK[BudZero / BudZKVM]
  State --> XD[Cross-domain / bridge state]
  State --> Apps[BNS · B.U.D. · Pollen · Hub · SocialFi · AI]
```

## 2. Consensus-domain izolasyonu

```mermaid
flowchart LR
  POW[Permissionless PoW] --> FA[DomainFinalityAdapter]
  POS[Permissionless PoS] --> FA
  BFT[Permissionless BFT] --> FA
  ZK[ZK proof domain] --> FA
  POA[Isolated PoA / KYC membership] --> PFA[PoA finality adapter]
  FA --> Settlement[Global settlement commitment]
  PFA --> Settlement
  Reg[Permissionless stake registry] -. never shared .-> POA
  PoAReg[Separate PoA membership registry] -. isolated .-> Reg
```

## 3. Transaction admission and V4 signing

```mermaid
sequenceDiagram
  participant C as Client
  participant R as RPC / P2P decoder
  participant M as Mempool
  participant E as Executor
  C->>C: Canonical V4 payload + signature
  C->>R: Transaction
  R->>R: signature_version == V4?
  R->>R: canonical hash + signature verify
  R->>M: admitted transaction
  M->>E: selected block transaction
  E->>E: nonce / balance / type rules
  E-->>C: state transition or typed rejection
```

## 4. Cross-domain bridge lifecycle

```mermaid
flowchart LR
  Lock[Lock on source domain] --> Event[Committed lock event]
  Event --> Proof[Relayer proof]
  Proof --> Verify[Header / receipt / MPT verification]
  Verify --> Mint[Mint on target]
  Mint --> Burn[Burn on target]
  Burn --> BurnProof[Verified burn event]
  BurnProof --> Unlock[Unlock on original source]
  Verify --> Replay[Replay / correlation / state-machine gates]
  Replay --> Mint
  Replay --> Unlock
```

## 5. EVM receipt verification path

```mermaid
flowchart TB
  Headers[Target header + confirmation chain] --> HC[Parent hash / height checks]
  Receipt[Receipt envelope bytes] --> RLP[Strict RLP decoder]
  Proof[MPT proof nodes + receipt key] --> MPT[Merkle-Patricia verifier]
  HC --> Root[receiptsRoot]
  Root --> MPT
  RLP --> Log[Status + emitter + topic0 + payload checks]
  MPT --> ReceiptValue[Committed receipt bytes]
  ReceiptValue --> RLP
  Log --> Deposit[Verified deposit facts]
```

## 6. Snapshot trust and schema migration

```mermaid
flowchart LR
  Live[AccountState] --> S4[Schema-4 canonical digest]
  S4 --> Sign[Manifest signer / signature]
  Sign --> Disk[(Snapshot storage)]
  Disk --> Load[Loader]
  Load --> Integrity[Digest + field manifest verify]
  Integrity --> Auth[External trust policy verify]
  Auth --> Restore[Restore state]
  Integrity --> Quarantine[Quarantine / fail-loud]
  Auth --> Quarantine
  Legacy[Schema-2/3 legacy import] --> Versioned[Version-specific digest]
  Versioned --> Integrity
```

## 7. Critical durability boundary

```mermaid
flowchart LR
  Input[Verified bridge / QC / finality input] --> Stage[Stage next state]
  Stage --> Batch[Atomic durable batch]
  Batch --> Flush[Apply + flush]
  Flush --> Publish[Publish in-memory state]
  Flush -->|failure| Stop[Fail-stop / operator recovery]
  Publish --> Next[Relay / finality continuation]
```

## 8. BudZero execution and proof boundary

```mermaid
flowchart LR
  Contract[Contract bytecode] --> VM[bud-vm execution]
  VM --> Trace[Execution trace]
  Trace --> Proof[bud-proof / STARK proof]
  Proof --> Verify[Budlum verifier]
  Verify --> Exec[Executor state transition]
  VM --> Host[Host calls: AI request / chain integration]
```

## 9. AI inference lifecycle

```mermaid
flowchart LR
  Model[Model registration] --> Request[Inference request + fee escrow]
  Request --> Verifiers[Permissionless AI verifiers]
  Verifiers --> Results[Signed result commitments]
  Results --> Threshold[Agreement / equivocation checks]
  Threshold --> Outcome[Finalized outcome + callback]
  Threshold --> Reclaim[Expired no-consensus fee reclaim]
```

## 10. B.U.D. storage lifecycle

```mermaid
flowchart LR
  Content[Content manifest + shards] --> Deal[Permissionless storage deal]
  Deal --> Operator[Storage operator]
  Operator --> Challenge[Interim retrieval challenge]
  Challenge --> Outcome[Challenge outcome]
  Gate[VerifyMerkle 64-depth gate] -. required for real Proof-of-Storage .-> Production[Production proof-of-storage]
```

## 11. Mainnet launch gates

```mermaid
flowchart TD
  V29[V29 signing V4] --> P2[P2 snapshot integrity]
  P2 --> V19[Critical durability]
  V19 --> Audit[External audit / bounty]
  Audit --> Fuzz[Long-running fuzz campaign]
  Fuzz --> HSM[Vendor-native HSM decision/test]
  HSM --> Ceremony[Genesis ceremony + bootnodes]
  Ceremony --> Freeze[Genesis hash freeze]
  Freeze --> Launch[Mainnet launch approval]
  ZKGate[VerifyMerkle soundness or explicit feature gate] --> Launch
```

## 12. CI and security gates

```mermaid
flowchart LR
  Push[Commit / PR] --> Fmt[Format + lint]
  Fmt --> Core[Core / BudZero tests]
  Core --> Invariants[BNS · B.U.D. · PoA invariant gates]
  Invariants --> Coverage[Coverage ratchet]
  Coverage --> Supply[deny · SBOM · secret scan · geiger · udeps]
  Supply --> Smoke[Docker / multinode smoke]
  Smoke --> CI[CI verdict]
  Fuzz[Fuzz quick + nightly campaigns] -. ongoing evidence .-> CI
```

## 13. Privacy layer — note lifecycle (D2)

```mermaid
flowchart LR
  Seed[Wallet seed] --> Derive[derive_spend_secret + derive_blinding]
  Derive --> Note[PrivateNoteInput / PrivateNoteOutput]
  Note --> Commit[PrivacyCommit opcode → Poseidon3]
  Commit --> Reg[L1NoteRegistry live_commitments]
  Note --> Null[NullifierCheck opcode → Poseidon2]
  Null --> Spent[spent_nullifiers set]
  Reg --> Transfer[PrivateTransferSubmit tx]
  Transfer --> Verify[SumConservation opcode]
  Transfer --> Apply[apply_transfer: remove commitment + insert nullifier]
  ViewKey[View key disclosure] -. selective .-> Audit[Auditor / authority]
  TEE[TEE opt-in] -. encrypt .-> Note
```

## 14. Wallet-core architecture

```mermaid
flowchart TD
  Entropy[CSPRNG entropy] --> BIP39[BIP39 mnemonic 12/24 words]
  BIP39 --> Seed[PBKDF2 → 32-byte seed]
  Seed --> SLIP10[SLIP-10 Ed25519 HD derivation]
  SLIP10 --> KeyPair[SigningKey + VerifyingKey]
  KeyPair --> Address[SHA3-256 → BudlumAddress]
  KeyPair --> Sign[V4 canonical signing]
  Seed --> Privacy[derive_spend_secret / derive_blinding]
  Seed --> ViewKey[derive_view_key]
  TEE[TeeRuntime opt-in] -. seal .-> Sign
  Zeroize[Zeroize on drop] -. cleanup .-> Seed
  Zeroize -. cleanup .-> BIP39
  Recovery[Social recovery guardians] -. restore .-> Seed
```

## 15. Governance lifecycle

```mermaid
flowchart LR
  Propose[Proposal submitted] --> Active[Active voting period]
  Active --> Vote[Stake-weighted votes for/against]
  Active --> Timelock[activation_epoch timelock]
  Vote --> Finalize[Epoch advance → finalize]
  Finalize -->|quorum met + majority| Passed[Passed]
  Finalize -->|quorum not met| Rejected[Rejected]
  Passed --> Execute[Execute governance action]
  Timelock --> Execute
  Execute --> Params[Update chain parameters]
  Execute --> BlockReward[Change block reward]
  Execute --> Constitution[Update constitution guardrails]
  Cancel[Proposal cancellation] -. owner only .-> Active
```

## 16. Tokenomics flow

```mermaid
flowchart TD
  Genesis[100M BUD genesis] --> Community[Community 10M]
  Genesis --> Liquidity[Liquidity 10M]
  Genesis --> Ecosystem[Ecosystem 20M]
  Genesis --> Team[Team 20M vesting cliff+linear]
  Genesis --> BurnReserve[Burn reserve 40M]
  BurnReserve --> TimedBurn[process_timed_burn epoch-triggered]
  TxnFee[Tx fee] --> FeeBurn[tx_fee_burn_ratio metabolic burn]
  TxnFee --> Proposer[Proposer tip]
  TxnFee --> Treasury[Treasury share]
  BlockReward[block_reward mint] --> Proposer2[Block producer]
  TimedBurn --> Sink[Burn sink — supply decreases]
  FeeBurn --> Sink
```

## 17. P2P network topology

```mermaid
flowchart TB
  Node[Budlum node] --> Gossip[Gossipsub topics]
  Gossip --> Blocks[Block announcements]
  Gossip --> Txs[Transaction relay]
  Gossip --> Finality[Finality certificates]
  Node --> Peers[PeerManager]
  Peers --> MaxPeers[MAX_PEERS = 50]
  Peers --> Subnet[max_peers_per_subnet /24 = 4]
  Peers --> Score[Reputation scoring]
  Score --> Ban[Ban threshold ≤ -100]
  Node --> Snap[Snapshot sync]
  Snap --> Chunks[MAX_SNAPSHOT_CHUNKS = 4096]
  Snap --> Concurrent[MAX_CONCURRENT_SNAPSHOTS = 10]
  Node --> Identity[Ed25519 identity key]
  Identity --> Auth[Peer authentication]
```

## 18. Permissionless registry architecture

```mermaid
flowchart LR
  Stake[Stake tx] --> Reg[PermissionlessRegistry]
  Reg --> Roles[RoleId: VALIDATOR · VERIFIER · RELAYER · PROVER · STORAGE_OPERATOR · AI_VERIFIER · ATTESTER · LUBOT_OPERATOR · CONTENT_VALIDATOR]
  Reg --> Slash[SlashingReport → slash]
  Slash --> DoubleSign[DoubleSign → 100%]
  Slash --> Liveness[LivenessFault → configurable]
  Slash --> Malicious[MaliciousBehaviour → 100%]
  Reg --> Unbond[Unbond tx → unbonding_queue]
  Unbond --> Epoch[Epoch advance → release]
  CrossRole[Cross-role slashing] -. slash one .-> AllRoles[All roles jailed]
```

## 19. PoA domain lifecycle

```mermaid
flowchart LR
  KYC[KYC / identity verification] --> Membership[PoaMembershipRegistry]
  Membership --> Admin[Admin approval]
  Admin --> Active[Active PoA member]
  Active --> Sign[Ed25519 finality signatures]
  Active --> Compliance[PoaComplianceRegistry]
  Compliance --> Screen[Address screening]
  Compliance --> Freeze[Asset freeze]
  Compliance --> TravelRule[Travel rule metadata hash]
  Compliance --> Audit[Append-only audit log]
  Isolation[PoA isolated from permissionless domains] -. no shared registry .-> Permissionless
```

## 20. Validator lifecycle

```mermaid
flowchart TD
  Genesis[Genesis config] --> Val[Validator created with keys]
  Stake[Stake tx] --> Active[Active validator]
  Active --> Propose[Block proposal via VRF]
  Active --> Finality[BLS finality signing]
  Active --> Slash[Slashing evidence]
  Slash --> Jailed[Jailed until epoch N]
  Jailed --> Release[Jail release]
  Release --> Active
  Unstake[Unstake tx] --> Unbonding[Unbonding queue]
  Unbonding --> Epoch[Epoch advance → release stake]
  Liveness[Missed epochs > threshold] --> LivenessSlash[Liveness report → slash]
```

## 21. Pollen data rights lifecycle

```mermaid
flowchart LR
  Asset[DataAsset registered] --> Grant[AccessGrant issued]
  Grant --> Grantee[Grantee address + scope + expiry]
  Asset --> Sale[SaleAuthorization]
  Sale --> Buyer[Buyer purchases access]
  Buyer --> Purchase[PollenPurchaseReceipt]
  Grant --> AI[AI inference request]
  AI --> Gate[Pollen data gate: valid grant required]
  Gate -->|grant valid| Allow[Allow data read]
  Gate -->|no grant| Deny[Deny — strict default-deny]
  Encrypt[EncryptionPolicy DAO-managed] -. parameters .-> Asset
  Revoke[Revoke grant/asset] -. owner only .-> Grant
```

## 22. Relayer policy layer

```mermaid
flowchart LR
  User[User intent] --> Intent[UserIntent signed]
  Intent --> Pool[Intent pool]
  Pool --> Solver[Solver bids]
  Solver --> Best[Best bid selection]
  Best --> Settle[IntentSettlement]
  Settle --> Execute[Execute settlement]
  Policy[PolicyEnvelope] --> FeeCap[Fee cap enforcement]
  Policy --> Deadline[Deadline validation]
  Policy --> Domain[Domain allowlist]
  Policy --> Replay[Replay nonce check]
  Slashing[Relayer slashing] --> Griefing[Griefing → 100%]
  Slashing --> FrontRunning[Front-running → 100%]
  Slashing --> WrongRelay[Wrong-relay → 100%]
```

## 23. Fee market (EIP-1559)

```mermaid
flowchart LR
  Block[Block N-1 base_fee] --> Calc[next_base_fee calculation]
  Calc --> Adjustment[±12.5% adjustment based on gas usage]
  Adjustment --> BaseFee[Block N base_fee]
  Tx[Transaction] --> Bid[FeeBid: max_fee + max_priority_fee]
  Bid --> Effective[effective_fee = min(max_fee, base_fee + priority)]
  Effective --> Check[effective_fee ≥ base_fee?]
  Check -->|yes| Accept[Accepted]
  Check -->|no| Reject[Rejected — underpriced]
  Accept --> Burn[base_fee burned]
  Accept --> Tip[priority_fee → proposer]
```

## 24. AI execution proof pipeline

```mermaid
flowchart TD
  Model[FixedPointMlpSpec] --> Host[Host eval_fixed_point_mlp i32 MAC]
  Host --> Output[Output limbs]
  Model --> Guest[build_matmul_guest_program BudZKVM instructions]
  Guest --> ProgramHash[program_hash_from_words]
  Model --> Weights[weights_digest SHA3-256]
  Weights --> Bytecode[Guest bytecode: Load + Mul + Add + ReLU + Poseidon + Halt]
  Bytecode --> Prove[prove_bytecode → STARK proof]
  Prove --> Envelope[ProofEnvelope postcard]
  Envelope --> Attach[AiAttachExecutionProof tx]
  Attach --> Verify[Structural verify + program_hash bind]
  Verify --> STARK[STARK verify via DefaultAdapter]
  STARK --> Finalize[try_finalize_with_proofs]
```

## 25. DeEd content manifest architecture

```mermaid
flowchart LR
  Content[Raw content bytes] --> Hash[ContentId = SHA3-256 domain-tagged]
  Content --> Shards[Off-chain sharding]
  Shards --> ShardRef[ShardRef: shard_id + size]
  Hash --> Manifest[ContentManifest: shards + metadata + owner]
  Manifest --> ManifestId[ManifestId = deterministic hash]
  ManifestId --> Chain[On-chain registration]
  Chain --> Deal[Storage deal per shard]
  Deal --> Operator[Storage operator bonds]
  Deal --> Challenge[Retrieval challenge]
  Challenge --> Proof[VerifyMerkle 64-depth proof]
  Roles[Permissionless roles: STORAGE_OPERATOR · ATTESTER] -. no whitelist .-> Deal
```

## 26. BNS (Budlum Name Service) lifecycle

```mermaid
flowchart LR
  Register[Register name 3-32 chars] --> Cost[Cost = base × multiplier × duration]
  Cost --> Owner[Owner address bound]
  Owner --> Resolve[resolve_content → address]
  Owner --> SetContent[set_content → CID/hash]
  Owner --> Transfer[Transfer to new owner]
  Owner --> Renew[Renew before expiry]
  Expiry[Expiry epoch reached] --> Grace[Grace period 3000 epochs]
  Grace -->|original owner| Renew2[Renew only by original owner]
  Grace -->|expired| Available[Name available for re-registration]
  Squat[Front-running squatting protection] -. grace period .-> Grace
```

## 27. SocialFi NFT lifecycle

```mermaid
flowchart LR
  Mint[Mint NFT owner-only] --> Metadata[CID + luminance=0]
  Metadata --> Luminance[update_luminance delta i128]
  Luminance --> Positive[Positive: amplify reach]
  Luminance --> Negative[Negative: reduce reach]
  Mint --> Transfer[Transfer to new owner]
  Mint --> Burn[Burn → CID returned]
  Registry[NftRegistry next_id auto-increment] --> Mint
  Guard[Luminance clamp i128 → safe range] --> Luminance
```

## 28. Hub app registry

```mermaid
flowchart LR
  Developer[Developer address] --> Register[register_app auto-increment ID]
  Register --> Manifest[AppManifest: URL + metadata]
  Manifest --> Update[update_app URL/manifest]
  Register --> SelfVerify[verify_app developer self-verify]
  SelfVerify --> Attested[developer_attested = true]
  Attested --> Verified[verified = true DAO override reserved]
  Root[Registry root hash] --> StateRoot[AccountState state_root]
  Audit[Attestation audit trail] --> SelfVerify
```

## 29. Mempool internals

```mermaid
flowchart TD
  Tx[Incoming transaction] --> Decode[V4 decode + signature verify]
  Decode --> Admit[Admission: nonce + balance + type rules]
  Admit --> Pool[Mempool pool max_size=20000]
  Pool --> PerSender[max_per_sender=100]
  Pool --> Evict[evict_lowest_fee when full]
  Pool --> RBF[Replace-By-Fee: higher fee replaces]
  Pool --> Dedup[Duplicate tx rejection]
  Pool --> Select[Block producer selects by fee priority]
  Select --> Block[Included in next block]
  Select --> Expire[Stale tx removed after N blocks]
```

## 30. Developer OS / SDK architecture

```mermaid
flowchart LR
  Manifest[DeveloperOsManifest deterministic] --> Project[Project ID + labels]
  Project --> DevNet[Local devnet topology]
  Project --> BudL[BudL package fixtures]
  Project --> Proof[Proof fixtures]
  Project --> Pollen[Pollen fixtures]
  Project --> Relayer[Relayer policy fixtures]
  Manifest --> Flags[SDK feature flags]
  Flags --> Offline[Offline default: no external network]
  Flags --> Safety[Safety fixtures: verified proof required]
  Safety --> NoMock[Pollen fixture cannot bypass AI grant]
  Project --> Traversal[Path traversal rejection]
```

## 31. Gateway — Atlas + Passport evidence

```mermaid
flowchart LR
  Address[BudlumAddress] --> Atlas[AtlasWalletContext]
  Atlas --> Account[Account state evidence]
  Atlas --> Pollen[Pollen lineage summary]
  Atlas --> Domain[Domain trace + wallet graph]
  Name[BNS name] --> Passport[DwebPassportProfile]
  Passport --> Evidence[EvidenceCard: verified/expired/pending]
  Passport --> Bundle[PassportProofBundle deterministic root]
  Bundle --> Warning[Warning hash only — no plaintext]
  Atlas --> RPC[bud_atlasGetWalletContext read-only]
  Passport --> RPC2[bud_passportGetProofBundle read-only]
  NoPlaintext[Endpoint never returns raw data] -. enforced .-> RPC
  NoPlaintext -. enforced .-> RPC2
```

## 32. Settlement commitment tree

```mermaid
flowchart TD
  Block[Block commitment] --> Roots[12+ root fields]
  Roots --> DomainReg[domain_registry_root]
  Roots --> Commitment[commitment_root]
  Roots --> Message[message_root]
  Roots --> Bridge[bridge_root]
  Roots --> Replay[replay_root]
  Roots --> Settlement[settlement_root]
  Roots --> Storage[storage_root]
  Roots --> AI[ai_root]
  Roots --> Pollen[pollen_root]
  DomainSep[BDLM_GLOBAL_BLOCK_V2 domain separation] --> Hash[GlobalBlockHeader hash]
  Proof[SettlementProofVerifier] --> Merkle[Merkle proof + domain/height/index check]
  Merkle --> Forge[expected_block_hash forgery gate]
```

## 33. Prover market — proof verification

```mermaid
flowchart LR
  Task[ProofTask created] --> Assign[Assigned to prover]
  Assign --> Prove[Prover generates STARK proof]
  Prove --> Receipt[ProofReceipt: task_id + prover + hash + reward]
  Receipt --> Verify[Verification: task_id + prover + epoch + hash + reward cap]
  Verify -->|valid| Complete[complete_task → reward committed]
  Verify -->|invalid| Reject[Reject — task stays active]
  Policy[First valid receipt wins] --> Complete
  Policy --> Duplicate[Identical duplicate → idempotent]
  Limits[Active tasks + pending receipts bounded] --> Task
```

## 34. Sovereign domain kit

```mermaid
flowchart LR
  Template[SovereignDomainTemplate] --> Class[SovereignClass enum]
  Class --> PoA[EnterprisePoa → requires PoA consensus]
  Class --> Custom[Custom class label validated]
  Template --> Compliance[ComplianceEvidence hash/root only]
  Compliance --> NoPII[No private KYC/passport on-chain]
  Template --> Lifecycle[Lifecycle: draft → active → retired]
  Lifecycle --> NoReactivate[Retired cannot re-activate]
  Template --> Audit[AuditExportBundle template + compliance root]
  Audit --> Bounded[Bounded height span]
```

## 35. Constitution engine

```mermaid
flowchart TD
  Guardrails[Hard guardrails immutable] --> NoWhitelist[No permissionless whitelist]
  Guardrails --> NoDecrypt[No AI read/decrypt override]
  Guardrails --> PoAIsolation[PoA domain isolation]
  Guardrails --> NoCustody[No private key custody]
  Guardrails --> EvidenceOnly[Evidence-only API]
  Params[Mutable bounded params] --> HaltMax[emergency_halt_max_epochs]
  Params --> PropMin[constitution_proposal_min_epochs]
  Governance[SetConstitutionParameter proposal] --> Params
  Governance -->|hard guardrail update| Reject[Fail-closed rejected]
  Root[Constitution root hash] --> StateRoot[AccountState state_root]
```

## 36. Mobile self-hosting profile

```mermaid
flowchart LR
  Profile[MobileNodeProfile] --> Power[PowerMode: battery/saver/performance]
  Power --> Battery[BatteryStatus validated]
  Profile --> Network[NetworkStatus: bandwidth + latency + NAT]
  Profile --> Storage[StorageStatus: capacity + availability]
  Network --> Relay[Relay address for NAT traversal]
  Storage --> Critical[Critical content requires paid replica]
  Profile --> Opportunistic[Opportunistic hosting — not always-on]
  Profile --> Scheduled[Scheduled replication windows]
  Validation[Impossible battery state rejected] --> Battery
  Validation[Zero bandwidth rejected] --> Network
```

## 37. Encryption DAO policy lifecycle

```mermaid
flowchart LR
  DAO[Governance proposal] --> SetPolicy[SetEncryptionPolicy action]
  SetPolicy --> Policy[EncryptionPolicy: version + suite + limits]
  Policy --> Active[active = true]
  Policy --> Deprecated[deprecated_after_block set]
  Policy --> MinKey[min_public_key_bytes enforced]
  Policy --> MaxGrant[max_grant_duration_blocks enforced]
  Policy --> NoDecrypt[No decrypt/key/read override fields]
  Asset[AssetEncryptionPolicy per-asset] --> Validate[validate_static: algorithm + key length + rotation]
  Validate --> Reject[EncryptionAlgorithm::None rejected]
  Root[Pollen root hash] --> StateRoot[AccountState state_root]
```

## 38. Pre-mortem security audit — attack graph

```mermaid
flowchart TD
  CIPat[CI PAT leak] --> MainBranch[main branch compromise]
  MainBranch --> CodeManip[Code manipulation]
  PKCS11[PKCS11 data object] --> BLSExtract[BLS key extraction]
  BLSExtract --> FinalityForge[Finality forge]
  BLSBias[BLS hash_to_g1 bias] --> FinalityManip[Finality manipulation]
  RPCNoAuth[RPC no auth] --> BridgeMint[Unauth bridge mint]
  BridgeNoPayload[Bridge mint no payload check] --> FundInflation[Fund inflation]
  SaturatingArith[saturating arithmetic] --> SilentLoss[Silent BUD loss]
  BlindingTrunc[Blinding truncation] --> PrivacyBreak[Privacy break]
  NullifierCollision[Nullifier collision] --> DoubleSpend[Double-spend]
  PoseidonDesync[Poseidon constants desync] --> AllProofsFail[All proofs rejected]
  SeedMemory[Seed in memory] --> TotalLoss[Total fund loss]
  C1 --> C1Fix[dual SHA3-256 LO/HI ✅]
  H1 --> H1Fix[Extractable=false ✅]
  R2 --> R2Fix[require_operator ✅]
  S1 --> S1Fix[register-based blinding ✅]
  B2 --> B2Fix[payload_hash verify ✅]
  E1 --> E1Fix[checked arithmetic ✅]
```
