# 数据库设计

## 表 `SATPOINT_TO_SEQUENCE_NUMBER`
### key: `SatPointValue` 转换成[u8, 44]
```rust

pub struct SatPoint {
  pub outpoint: OutPoint,
  pub offset: u64,
}

pub struct OutPoint {
    /// The referenced transaction's txid.
    pub txid: Txid,
    /// The index of the referenced output in its transaction's vout.
    pub vout: u32,
}

```


## value: `sequence_numbers` u32

## 表 `SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY`
### key: `sequence_number` u32
### value: `InscriptionEntry`

```rust
pub(crate) type InscriptionEntryValue = (
  u16,                // charms
  u64,                // fee
  u32,                // height
  InscriptionIdValue, // inscription id
  i32,                // inscription number
  Vec<u32>,           // parents
  Option<u64>,        // sat
  u32,                // sequence number
  u32,                // timestamp
);
```

## value: `sequence_numbers` u32

## 表 `SAT_TO_SEQUENCE_NUMBER`
### key: `sat` u64
### value: `sequence_number` u32

```rust
pub struct Sat(pub u64);
```

## 表 `SEQUENCE_NUMBER_TO_CHILDREN`
### key: `sequence_number` u32
### value: `sequence_number` u32

```rust
    sequence_number_to_children.insert(parent_sequence_number, sequence_number)?;
```

## 表 `CONTENT_TYPE_TO_COUNT`
### key: `content_type` Option<&[u8]>
### value: `count` u64

```rust
  //  content_type: http_content_type
```

## 表 `HEIGHT_TO_BLOCK_HEADER`
### key: `height` u32
### value: `HeaderValue`  [u8; 80]

```rust
  pub struct Header {
    /// Block version, now repurposed for soft fork signalling.
    pub version: Version,
    /// Reference to the previous block in the chain.
    pub prev_blockhash: BlockHash,
    /// The root hash of the merkle tree of transactions in the block.
    pub merkle_root: TxMerkleNode,
    /// The timestamp of the block, as claimed by the miner.
    pub time: u32,
    /// The target value below which the blockhash must lie.
    pub bits: CompactTarget,
    /// The nonce, selected to obtain a low enough blockhash.
    pub nonce: u32,
}

// 将区块头压缩成字节流
impl_consensus_encoding!(Header, version, prev_blockhash, merkle_root, time, bits, nonce);
```

## 表 `HEIGHT_TO_LAST_SEQUENCE_NUMBER`
### key: `height` u32
### value: `sequence_number`  u32

```rust
height_to_last_sequence_number
    .insert(&self.height, inscription_updater.next_sequence_number)?;
```

## 表 `HOME_INSCRIPTIONS`
### key: `sequence_number` u32
### value: `InscriptionIdValue` (u128, u128, u32)

```rust
pub struct InscriptionId {
    pub txid: Txid,
    pub index: u32,
}

// u128(txid 前半部分), u128(txid 后半部分), u32(当前区块第几个交易: index)
fn store(self) -> Self::Value {
    let txid_entry = self.txid.store();
    let little_end = u128::from_le_bytes(txid_entry[..16].try_into().unwrap());
    let big_end = u128::from_le_bytes(txid_entry[16..].try_into().unwrap());
    (little_end, big_end, self.index)
}
```

## 表 `INSCRIPTION_ID_TO_SEQUENCE_NUMBER`
### key: `InscriptionIdValue` (u128, u128, u32)
### value: `sequence_number` u32

```rust

```

## 表 `INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER`
### key: `inscription_number` i32
### value: `sequence_number` u32

```rust
let inscription_number = if cursed {
    let number: i32 = self.cursed_inscription_count.try_into().unwrap();
    self.cursed_inscription_count += 1; // 可恨的符文数
    -(number + 1)
} else {
    let number: i32 = self.blessed_inscription_count.try_into().unwrap();
    self.blessed_inscription_count += 1; // 被祝福的符文数
    number
};
```

## 表 `OUTPOINT_TO_RUNE_BALANCES`
### key: `OutPointValue` [u8; 36]
### value: `balances` [u8]

```rust
pub struct OutPoint {
    /// The referenced transaction's txid.
    pub txid: Txid,
    /// The index of the referenced output in its transaction's vout.
    pub vout: u32,
}
```

## 表 `OUTPOINT_TO_SAT_RANGES`
### key: `OutPointValue` [u8; 36]
### value: `sat_ranges` [u8]

```rust
pub struct SatPoint {
    pub outpoint: OutPoint,
    pub offset: u64,
}

let (start, end) = SatRange::load(chunk.try_into().unwrap());
if start <= sat && sat < end {
    return Ok(Some(SatPoint {
        outpoint: Entry::load(*key.value()),
        offset: offset + sat - start, // 当前outpoint的偏移量，将两个数字存到一个变量中: sat_ranges
    }));
}
```

## 表 `OUTPOINT_TO_VALUE` 用于标记outpoint是否已存在于数据库
### key: `OutPointValue` [u8; 36]
### value: `value` [u8]

```rust

```

## 表 `RUNE_ID_TO_RUNE_ENTRY`
### key: `RuneIdValue` (u64, u32)
### value: `RuneEntryValue` type RuneEntryValue

```rust
pub struct RuneId {
  pub block: u64,
  pub tx: u32,
}

pub(super) type RuneEntryValue = (
    u64,                     // block
    u128,                    // burned
    u8,                      // divisibility
    (u128, u128),            // etching
    u128,                    // mints
    u64,                     // number
    u128,                    // premine
    (u128, u32),             // spaced rune
    Option<char>,            // symbol
    Option<TermsEntryValue>, // terms
    u64,                     // timestamp
    bool,                    // turbo
);
```

## 表 `RUNE_TO_RUNE_ID`
### key: `rune` u128
### value: `RuneIdValue` (u64, u32)

```rust
pub struct Rune(pub u128);
```

## 表 `SAT_TO_SATPOINT`
### key: `sat` u64
### value: `SatPointValue` [u8; 44]

```rust
pub struct Sat(pub u64);
```

## 表 `SEQUENCE_NUMBER_TO_RUNE_ID`
### key: `sequence_number` u32
### value: `RuneIdValue` (u64, u32)

```rust

```

## 表 `SEQUENCE_NUMBER_TO_SATPOINT`
### key: `sequence_number` u32
### value: `SatPointValue` [u8; 44]

```rust

```

## 表 `STATISTIC_TO_COUNT`
### key: `statistic` u64
### value: `count` u64

```rust
pub(crate) enum Statistic {
  Schema = 0,
  BlessedInscriptions = 1,
  Commits = 2,
  CursedInscriptions = 3,
  IndexRunes = 4,
  IndexSats = 5,
  LostSats = 6,
  OutputsTraversed = 7,
  ReservedRunes = 8,
  Runes = 9,
  SatRanges = 10,
  UnboundInscriptions = 11,
  IndexTransactions = 12,
  IndexSpentSats = 13,
  InitialSyncTime = 14,
}
```

## 表 `TRANSACTION_ID_TO_RUNE`
### key: `TxidValue` [u8; 32]
### value: `rune` u128

```rust
// TxidValue: txid
```

## 表 `TRANSACTION_ID_TO_TRANSACTION`
### key: `TxidValue` [u8; 32]
### value: `transaction` &[u8] 序列化后的交易

```rust
pub struct Transaction {
    /// The protocol version, is currently expected to be 1 or 2 (BIP 68).
    pub version: i32,
    /// Block height or timestamp. Transaction cannot be included in a block until this height/time.
    ///
    /// ### Relevant BIPs
    ///
    /// * [BIP-65 OP_CHECKLOCKTIMEVERIFY](https://github.com/bitcoin/bips/blob/master/bip-0065.mediawiki)
    /// * [BIP-113 Median time-past as endpoint for lock-time calculations](https://github.com/bitcoin/bips/blob/master/bip-0113.mediawiki)
    pub lock_time: absolute::LockTime,
    /// List of transaction inputs.
    pub input: Vec<TxIn>,
    /// List of transaction outputs.
    pub output: Vec<TxOut>,
}
```

## 表 `WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP`
### key: `height` u32
### value: `timestamp` u128

```rust
wtx
  .open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?
  .insert(
    &self.height,
    &SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)?
      .as_millis(),
  )?;
```
