use crypto::rust::{
    hash::blake2b256::Blake2b256,
    private_key::PrivateKey,
    signatures::{secp256k1::Secp256k1, signatures_alg::SignaturesAlg},
};
use models::casper::v1::deploy_response::Message as DeployResponseMessage;
use models::casper::v1::deploy_service_client::DeployServiceClient;
use models::casper::DeployDataProto;
use prost::Message;

pub struct F1r3flyApi<'a> {
    signing_key: PrivateKey,
    node_host: &'a str,
    grpc_port: u16,
}

impl<'a> F1r3flyApi<'a> {
    pub fn new(signing_key: Vec<u8>, node_host: &'a str, grpc_port: u16) -> Self {
        F1r3flyApi {
            signing_key: PrivateKey::new(signing_key),
            node_host,
            grpc_port,
        }
    }

    pub async fn deploy(
        &self,
        rho_code: &str,
        use_bigger_rhlo_price: bool,
        language: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        //  let max_rholang_in_logs = 2000;
        //  debug!("Rholang code {}", if rho_code.len() > max_rholang_in_logs { &rho_code[..max_rholang_in_logs] } else { rho_code });

        let phlo_limit: i64 = if use_bigger_rhlo_price {
            5_000_000_000
        } else {
            50_000
        };

        let mut deployment = DeployDataProto {
            term: rho_code.to_string(),
            timestamp: 0,
            phlo_price: 1,
            phlo_limit,
            shard_id: "root".to_string(),
            language: language.to_string(),
            ..Default::default()
        };

        self.sign_deploy(&mut deployment)?;

        let mut deploy_service_client =
            DeployServiceClient::connect(format!("http://{}:{}/", self.node_host, self.grpc_port)).await?;

        let deploy_response = deploy_service_client.do_deploy(deployment).await?;
        let deploy_message: &DeployResponseMessage = deploy_response
            .get_ref()
            .message
            .as_ref()
            .expect("Deploy result not found");

        let deploy_result = match deploy_message {
            DeployResponseMessage::Error(service_error) => {
                return Err(Box::new(service_error.clone()));
            }
            DeployResponseMessage::Result(result) => result,
        };

        Ok(deploy_result.clone())

        //  if let Some(error) = deploy_response.get_ref().error.as_ref() {
        //      return Err(Box::new(ServiceError::new(error.clone())));
        //  }

        //  let deploy_result = &deploy_response.get_ref().result;
        //  let deploy_id = deploy_result.split("DeployId is: ").nth(1).unwrap_or("");

        //  let propose_response = self.propose_service.propose(Request::new(ProposeQuery { is_async: false })).await?;
        //  if let Some(error) = propose_response.get_ref().error.as_ref() {
        //      return Err(Box::new(ServiceError::new(error.clone())));
        //  }

        //  let b64 = hex::decode(deploy_id)?;
        //  let find_response = self.deploy_service.find_deploy(Request::new(FindDeployQuery { deploy_id: b64 })).await?;
        //  if let Some(error) = find_response.get_ref().error.as_ref() {
        //      return Err(Box::new(ServiceError::new(error.clone())));
        //  }

        //  let block_hash = &find_response.get_ref().block_info.as_ref().unwrap().block_hash;
        //  debug!("Block Hash {}", block_hash);

        //  let is_finalized_response = self.deploy_service.is_finalized(Request::new(IsFinalizedQuery { hash: block_hash.clone() })).await?;
        //  if let Some(error) = is_finalized_response.get_ref().error.as_ref() || !is_finalized_response.get_ref().is_finalized {
        //      return Err(Box::new(ServiceError::new(error.clone())));
        //  }

        //  Ok(block_hash.clone())
    }

    //  pub async fn find_data_by_name(&self, expr: &str) -> Result<Vec<RhoTypesPar>, Box<dyn std::error::Error>> {
    //      info!("Find data by name {}", expr);

    //      let par = RhoTypesPar {
    //          exprs: vec![RhoTypesExpr { g_string: expr.to_string(), ..Default::default() }],
    //          ..Default::default()
    //      };

    //      let request = DataAtNameQuery {
    //          name: Some(par),
    //          depth: 50,
    //          ..Default::default()
    //      };

    //      let response = self.deploy_service.listen_for_data_at_name(Request::new(request)).await?;
    //      if let Some(error) = response.get_ref().error.as_ref() {
    //          return Err(Box::new(ServiceError::new(error.clone())));
    //      }

    //      let payload = response.get_ref().payload.as_ref().unwrap();
    //      if payload.length == 0 {
    //          return Err(Box::new(NoDataByPath::new(expr.to_string())));
    //      }

    //      Ok(payload.block_info[0].post_block_data.clone())
    //  }

    //  pub async fn get_data_at_block_by_name(&self, block_hash: &str, expr: &str) -> Result<Vec<RhoTypesPar>, Box<dyn std::error::Error>> {
    //      info!("Get data at block {} by name {}", block_hash, expr);

    //      let par = RhoTypesPar {
    //          exprs: vec![RhoTypesExpr { g_string: expr.to_string(), ..Default::default() }],
    //          ..Default::default()
    //      };

    //      let request = DataAtNameByBlockQuery {
    //          block_hash: block_hash.to_string(),
    //          par: Some(par),
    //          ..Default::default()
    //      };

    //      let response = self.deploy_service.get_data_at_name(Request::new(request)).await?;
    //      if let Some(error) = response.get_ref().error.as_ref() {
    //          return Err(Box::new(ServiceError::new(error.clone())));
    //      }

    //      Ok(response.get_ref().payload.as_ref().unwrap().par.clone())
    //  }

    fn sign_deploy(&self, deploy: &mut DeployDataProto) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = deploy.encode_to_vec();
        let hashed = Blake2b256::hash(encoded);

        let secp256k1 = Secp256k1;
        let signature = secp256k1.sign(&hashed, &self.signing_key.bytes);
        let pub_key = secp256k1.to_public(&self.signing_key);

        deploy.sig_algorithm = "secp256k1".to_string();
        deploy.sig = signature;
        deploy.deployer = pub_key.bytes;

        Ok(())
    }
}
