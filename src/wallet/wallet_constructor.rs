use super::*;

#[derive(Clone)]
pub(crate) struct WalletConstructor {
  ord_client: reqwest::blocking::Client,
  name: String,
  no_sync: bool,
  rpc_url: Url,
  settings: Settings,
}

impl WalletConstructor {
  pub(crate) fn construct(
    name: String,
    no_sync: bool,
    settings: Settings,
    rpc_url: Url,
  ) -> Result<Wallet> {
    // 构建ord rpc request headers
    let mut headers = HeaderMap::new();
    headers.insert(
      header::ACCEPT,
      header::HeaderValue::from_static("application/json"),
    );

    if let Some((username, password)) = settings.credentials() {
      let credentials =
        base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
      headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Basic {credentials}")).unwrap(),
      );
    }

    Self {
      ord_client: reqwest::blocking::ClientBuilder::new()
        .default_headers(headers.clone())
        .build()?,
      name,
      no_sync,
      rpc_url,
      settings,
    }
    .build()
  }

  pub(crate) fn build(self) -> Result<Wallet> {
    // 打开钱包数据库
    let database = Wallet::open_database(&self.name, &self.settings)?;

    // 构建bitcoin rpc client，同时检查bitcoin core 钱包版本，加载钱包, 检查UTXO描述符
    /*
    UTXO 描述符提供了一种标准化的方法来描述比特币钱包需要处理的各种类型的输出。这使得钱包软件可以更容易地理解和使用这些输出，
    无论它们是 P2PKH（Pay-to-Public-Key-Hash）、P2SH（Pay-to-Script-Hash）、P2WPKH（Pay-to-Witness-Public-Key-Hash）还是其他类型的输出。
    
    如：UTXO 描述符 
    tr([423050f1/86h/1h/0h/1/1]bc4d4f74692678d4246a737d2654ea088ba879d5204e6a76200a54b679c9c031)#gvghjmsd

    这个描述符的各个部分的含义如下：
    - tr：这是描述符的类型，表示这是一个 Taproot 输出。Taproot 是比特币的一种新的地址类型，它使用了 Schnorr 签名和 MAST（Merklized Abstract Syntax Trees）技术，可以提供更好的隐私和更高的灵活性。
    - 423050f1/86h/1h/0h/1/1]：这是一个硬币类型路径（coin type path），它描述了如何从主公钥（master public key）派生出子公钥。这个路径遵循了 BIP32 提议的规定，86h/1h/0h/1/1 表示这是从主公钥派生出的特定路径的公钥。
    - bc4d4f74692678d4246a737d2654ea088ba879d5204e6a76200a54b679c9c031：这是一个公钥，它是 Taproot 地址的一部分。
    - #gvghjmsd：这是一个校验和，用于检查描述符是否被正确地输入或复制。
     */
    let bitcoin_client = {
      let client =
        Wallet::check_version(self.settings.bitcoin_rpc_client(Some(self.name.clone()))?)?;

      if !client.list_wallets()?.contains(&self.name) {
        client.load_wallet(&self.name)?;
      }

      Wallet::check_descriptors(&self.name, client.list_descriptors(None)?.descriptors)?;

      client
    };

    let chain_block_count = bitcoin_client.get_block_count().unwrap() + 1;

    if !self.no_sync {
      for i in 0.. { // 无限循环，直到同步成功，或者尝试次数超过20次
        let response = self.get("/blockcount")?;

        if response
          .text()?
          .parse::<u64>()
          .expect("wallet failed to talk to server. Make sure `ord server` is running.")
          >= chain_block_count
        {
          break;
        } else if i == 20 {
          bail!("wallet failed to synchronize with `ord server` after {i} attempts");
        }
        std::thread::sleep(Duration::from_millis(50));
      }
    }

    // 获取所有UTXO，包含已锁定的UTXO
    let mut utxos = Self::get_utxos(&bitcoin_client)?;
    let locked_utxos = Self::get_locked_utxos(&bitcoin_client)?;
    utxos.extend(locked_utxos.clone());

    let output_info = self.get_output_info(utxos.clone().into_keys().collect())?;

    // 获取所有的铭文
    let inscriptions = output_info
      .iter()
      .flat_map(|(_output, info)| info.inscriptions.clone())
      .collect::<Vec<InscriptionId>>();

    // 从ord server获取铭文信息
    let (inscriptions, inscription_info) = self.get_inscriptions(&inscriptions)?;

    // 获取ord server的rune_index和sat_index
    let status = self.get_server_status()?;

    Ok(Wallet {
      bitcoin_client,
      database,
      has_rune_index: status.rune_index,
      has_sat_index: status.sat_index,
      inscription_info,
      inscriptions,
      locked_utxos,
      ord_client: self.ord_client,
      output_info,
      rpc_url: self.rpc_url,
      settings: self.settings,
      utxos,
    })
  }

  fn get_output_info(&self, outputs: Vec<OutPoint>) -> Result<BTreeMap<OutPoint, api::Output>> {
    let response = self.post("/outputs", &outputs)?; // todo: houfa 后面调查

    if !response.status().is_success() {
      bail!("wallet failed get outputs: {}", response.text()?);
    }

    let output_info: BTreeMap<OutPoint, api::Output> = outputs
      .into_iter()
      .zip(serde_json::from_str::<Vec<api::Output>>(&response.text()?)?)
      .collect();

    for (output, info) in &output_info {
      if !info.indexed {
        bail!("output in wallet but not in ord server: {output}");
      }
    }

    Ok(output_info)
  }

  fn get_inscriptions(
    &self,
    inscriptions: &Vec<InscriptionId>,
  ) -> Result<(
    BTreeMap<SatPoint, Vec<InscriptionId>>,
    BTreeMap<InscriptionId, api::Inscription>,
  )> {
    let response = self.post("/inscriptions", inscriptions)?;

    if !response.status().is_success() {
      bail!("wallet failed get inscriptions: {}", response.text()?);
    }

    let mut inscriptions = BTreeMap::new();
    let mut inscription_infos = BTreeMap::new();
    for info in serde_json::from_str::<Vec<api::Inscription>>(&response.text()?)? {
      inscriptions
        .entry(info.satpoint)
        .or_insert_with(Vec::new)
        .push(info.id);

      inscription_infos.insert(info.id, info);
    }

    Ok((inscriptions, inscription_infos))
  }

  fn get_utxos(bitcoin_client: &Client) -> Result<BTreeMap<OutPoint, TxOut>> {
    Ok(
      bitcoin_client
        .list_unspent(None, None, None, None, None)?
        .into_iter()
        .map(|utxo| {
          let outpoint = OutPoint::new(utxo.txid, utxo.vout);
          let txout = TxOut {
            script_pubkey: utxo.script_pub_key,
            value: utxo.amount.to_sat(),
          };

          (outpoint, txout)
        })
        .collect(),
    )
  }

  fn get_locked_utxos(bitcoin_client: &Client) -> Result<BTreeMap<OutPoint, TxOut>> {
    #[derive(Deserialize)]
    pub(crate) struct JsonOutPoint {
      txid: Txid,
      vout: u32,
    }

    let outpoints = bitcoin_client.call::<Vec<JsonOutPoint>>("listlockunspent", &[])?;

    let mut utxos = BTreeMap::new();

    for outpoint in outpoints {
      let txout = bitcoin_client
        .get_raw_transaction(&outpoint.txid, None)?
        .output
        .get(TryInto::<usize>::try_into(outpoint.vout).unwrap())
        .cloned()
        .ok_or_else(|| anyhow!("Invalid output index"))?;

      utxos.insert(OutPoint::new(outpoint.txid, outpoint.vout), txout);
    }

    Ok(utxos)
  }

  fn get_server_status(&self) -> Result<api::Status> {
    let response = self.get("/status")?;

    if !response.status().is_success() {
      bail!("could not get status: {}", response.text()?)
    }

    Ok(serde_json::from_str(&response.text()?)?)
  }

  pub fn get(&self, path: &str) -> Result<reqwest::blocking::Response> {
    self
      .ord_client
      .get(self.rpc_url.join(path)?)
      .send()
      .map_err(|err| anyhow!(err))
  }

  pub fn post(&self, path: &str, body: &impl Serialize) -> Result<reqwest::blocking::Response> {
    self
      .ord_client
      .post(self.rpc_url.join(path)?)
      .json(body)
      .header(reqwest::header::ACCEPT, "application/json")
      .send()
      .map_err(|err| anyhow!(err))
  }
}
