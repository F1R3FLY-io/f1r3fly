use super::contract_call::ContractCall;
use super::dispatch::RhoDispatch;
use super::pretty_printer::PrettyPrinter;
use super::registry::registry::Registry;
use super::rho_runtime::RhoISpace;
use super::rho_type::{
    RhoBoolean, RhoByteArray, RhoDeployerId, RhoName, RhoNumber, RhoString, RhoUri,
};
use super::util::rev_address::RevAddress;
use crypto::rust::hash::blake2b256::Blake2b256;
use crypto::rust::hash::keccak256::Keccak256;
use crypto::rust::hash::sha_256::Sha256Hasher;
use crypto::rust::public_key::PublicKey;
use crypto::rust::signatures::ed25519::Ed25519;
use crypto::rust::signatures::secp256k1::Secp256k1;
use crypto::rust::signatures::signatures_alg::SignaturesAlg;
use k256::{
    ecdsa::{signature::Signer, Signature, SigningKey},
    elliptic_curve::generic_array::GenericArray,
};
use models::rhoapi::expr::ExprInstance;
use models::rhoapi::g_unforgeable::UnfInstance::GPrivateBody;
use models::rhoapi::Expr;
use models::rhoapi::{Bundle, GPrivate, GUnforgeable, ListParWithRandom, Par, Var};
use models::rust::casper::protocol::casper_message::BlockMessage;
use models::rust::rholang::implicits::single_expr;
use models::rust::utils::{new_gbool_par, new_gbytearray_par, new_gsys_auth_token_par};
use models::Byte;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/SystemProcesses.scala
// NOTE: Not implementing Logger
pub type RhoSysFunction = Box<dyn Fn(Vec<ListParWithRandom>) -> Pin<Box<dyn Future<Output = ()>>>>;
pub type RhoDispatchMap = Arc<RwLock<HashMap<i64, RhoSysFunction>>>;
pub type Name = Par;
pub type Arity = i32;
pub type Remainder = Option<Var>;
pub type BodyRef = i64;
pub type Contract = dyn Fn(Vec<ListParWithRandom>) -> ();

#[derive(Clone)]
pub struct InvalidBlocks {
    pub invalid_blocks: Arc<RwLock<Par>>,
}

impl InvalidBlocks {
    pub fn new() -> Self {
        InvalidBlocks {
            invalid_blocks: Arc::new(RwLock::new(Par::default())),
        }
    }

    pub fn set_params(&self, invalid_blocks: Par) -> () {
        let mut lock: RwLockWriteGuard<Par> = self.invalid_blocks.write().unwrap();

        *lock = invalid_blocks;
    }
}

pub fn byte_name(b: Byte) -> Par {
    Par::default().with_unforgeables(vec![GUnforgeable {
        unf_instance: Some(GPrivateBody(GPrivate { id: vec![b] })),
    }])
}

pub struct FixedChannels;

impl FixedChannels {
    pub fn stdout() -> Par {
        byte_name(0)
    }

    pub fn stdout_ack() -> Par {
        byte_name(1)
    }

    pub fn stderr() -> Par {
        byte_name(2)
    }

    pub fn stderr_ack() -> Par {
        byte_name(3)
    }

    pub fn ed25519_verify() -> Par {
        byte_name(4)
    }

    pub fn sha256_hash() -> Par {
        byte_name(5)
    }

    pub fn keccak256_hash() -> Par {
        byte_name(6)
    }

    pub fn blake2b256_hash() -> Par {
        byte_name(7)
    }

    pub fn secp256k1_verify() -> Par {
        byte_name(8)
    }

    pub fn get_block_data() -> Par {
        byte_name(10)
    }

    pub fn get_invalid_blocks() -> Par {
        byte_name(11)
    }

    pub fn rev_address() -> Par {
        byte_name(12)
    }

    pub fn deployer_id_ops() -> Par {
        byte_name(13)
    }

    pub fn reg_lookup() -> Par {
        byte_name(14)
    }

    pub fn reg_insert_random() -> Par {
        byte_name(15)
    }

    pub fn reg_insert_signed() -> Par {
        byte_name(16)
    }

    pub fn reg_ops() -> Par {
        byte_name(17)
    }

    pub fn sys_authtoken_ops() -> Par {
        byte_name(18)
    }
}

pub struct BodyRefs;

impl BodyRefs {
    pub const STDOUT: i64 = 0;

    pub const STDOUT_ACK: i64 = 1;

    pub const STDERR: i64 = 2;

    pub const STDERR_ACK: i64 = 3;

    pub const ED25519_VERIFY: i64 = 4;

    pub const SHA256_HASH: i64 = 5;

    pub const KECCAK256_HASH: i64 = 6;

    pub const BLAKE2B256_HASH: i64 = 7;

    pub const SECP256K1_VERIFY: i64 = 9;

    pub const GET_BLOCK_DATA: i64 = 11;

    pub const GET_INVALID_BLOCKS: i64 = 12;

    pub const REV_ADDRESS: i64 = 13;

    pub const DEPLOYER_ID_OPS: i64 = 14;

    pub const REG_OPS: i64 = 15;

    pub const SYS_AUTHTOKEN_OPS: i64 = 16;
}

#[derive(Clone)]
pub struct ProcessContext {
    pub space: RhoISpace,
    pub dispatcher: RhoDispatch,
    pub block_data: Arc<RwLock<BlockData>>,
    pub invalid_blocks: InvalidBlocks,
    pub system_processes: SystemProcesses,
}

impl ProcessContext {
    pub fn create(
        space: RhoISpace,
        dispatcher: RhoDispatch,
        block_data: Arc<RwLock<BlockData>>,
        invalid_blocks: InvalidBlocks,
    ) -> Self {
        ProcessContext {
            space: space.clone(),
            dispatcher: dispatcher.clone(),
            block_data: block_data.clone(),
            invalid_blocks,
            system_processes: SystemProcesses::create(dispatcher, space, block_data),
        }
    }
}

pub struct Definition {
    pub urn: String,
    pub fixed_channel: Name,
    pub arity: Arity,
    pub body_ref: BodyRef,
    pub handler: Box<
        dyn FnMut(
            ProcessContext,
        )
            -> Box<dyn Fn(Vec<ListParWithRandom>) -> Pin<Box<dyn Future<Output = ()>>>>,
    >,
    pub remainder: Remainder,
}

impl Definition {
    pub fn new(
        urn: String,
        fixed_channel: Name,
        arity: Arity,
        body_ref: BodyRef,
        handler: Box<
            dyn FnMut(
                ProcessContext,
            )
                -> Box<dyn Fn(Vec<ListParWithRandom>) -> Pin<Box<dyn Future<Output = ()>>>>,
        >,
        remainder: Remainder,
    ) -> Self {
        Definition {
            urn,
            fixed_channel,
            arity,
            body_ref,
            handler,
            remainder,
        }
    }

    pub fn to_dispatch_table(
        &mut self,
        context: ProcessContext,
    ) -> (
        BodyRef,
        Box<dyn Fn(Vec<ListParWithRandom>) -> Pin<Box<dyn Future<Output = ()>>>>,
    ) {
        (self.body_ref, (self.handler)(context))
    }

    pub fn to_urn_map(&self) -> (String, Par) {
        let bundle: Par = Par::default().with_bundles(vec![Bundle {
            body: Some(self.fixed_channel.clone()),
            write_flag: true,
            read_flag: false,
        }]);

        (self.urn.clone(), bundle)
    }

    pub fn to_proc_defs(&self) -> (Name, Arity, Remainder, BodyRef) {
        (
            self.fixed_channel.clone(),
            self.arity,
            self.remainder.clone(),
            self.body_ref.clone(),
        )
    }
}

pub struct BlockData {
    pub time_stamp: i64,
    pub block_number: i64,
    pub sender: PublicKey,
    pub seq_num: i32,
}

impl BlockData {
    pub fn empty() -> Self {
        BlockData {
            block_number: 0,
            sender: PublicKey::from_bytes(&hex::decode("00").unwrap()),
            seq_num: 0,
            time_stamp: 0,
        }
    }

    pub fn from_block(template: &BlockMessage) -> Self {
        BlockData {
            time_stamp: template.header.timestamp,
            block_number: template.body.state.block_number,
            sender: PublicKey::from_bytes(&template.sender),
            seq_num: template.seq_num,
        }
    }
}

#[derive(Clone)]
pub struct SystemProcesses {
    pub dispatcher: RhoDispatch,
    pub space: RhoISpace,
    pub block_data: Arc<RwLock<BlockData>>,
    pretty_printer: PrettyPrinter,
}

impl SystemProcesses {
    fn create(
        dispatcher: RhoDispatch,
        space: RhoISpace,
        block_data: Arc<RwLock<BlockData>>,
    ) -> Self {
        SystemProcesses {
            dispatcher: dispatcher.clone(),
            space,
            block_data,
            pretty_printer: PrettyPrinter::new(),
        }
    }

    fn is_contract_call(&self) -> ContractCall {
        ContractCall {
            space: self.space.clone(),
            dispatcher: self.dispatcher.clone(),
        }
    }

    async fn verify_signature_contract(
        &self,
        contract_args: Vec<ListParWithRandom>,
        name: &str,
        algorithm: Box<dyn SignaturesAlg>,
    ) -> () {
        if let Some((produce, vec)) = self.is_contract_call().unapply(contract_args) {
            if let [data, signature, pub_key, ack] = &vec[..] {
                if let (Some(data_bytes), Some(signature_bytes), Some(pub_key_bytes)) = (
                    RhoByteArray::unapply(&data),
                    RhoByteArray::unapply(&signature),
                    RhoByteArray::unapply(&pub_key),
                ) {
                    let verified = algorithm.verify(&data_bytes, &signature_bytes, &pub_key_bytes);
                    if let Err(e) = produce(
                        vec![Par::default().with_exprs(vec![RhoBoolean::create_expr(verified)])],
                        ack.clone(),
                    )
                    .await
                    {
                        eprintln!("Error producing result: {:?}", e);
                    }
                } else {
                    panic!("{} expects data, signature, public key (all as byte arrays), and an acknowledgement channel", name)
                }
            } else {
                panic!("{} expects data, signature, public key (all as byte arrays), and an acknowledgement channel", name)
            }
        } else {
            panic!("{} expects data, signature, public key (all as byte arrays), and an acknowledgement channel", name)
        }
    }

    async fn hash_contract(
        &self,
        contract_args: Vec<ListParWithRandom>,
        name: &str,
        algorithm: Box<dyn Fn(Vec<u8>) -> Vec<u8>>,
    ) -> () {
        if let Some((produce, vec)) = self.is_contract_call().unapply(contract_args) {
            if vec.len() == 2 {
                if let (Some(input), Some(ack)) =
                    (vec.get(0).and_then(RhoByteArray::unapply), vec.get(1))
                {
                    let hash = algorithm(input);
                    if let Err(e) = produce(vec![RhoByteArray::create_par(hash)], ack.clone()).await
                    {
                        panic!("Error producing result named {}: {:?}", name, e);
                    }
                } else {
                    panic!("{} expects a byte array and return channel", name)
                }
            } else {
                panic!("{} expects a byte array and return channel", name)
            }
        } else {
            panic!("{} expects a byte array and return channel", name)
        }
    }

    fn print_std_out(&self, s: &str) -> () {
        println!("{}", s);
    }

    fn print_std_err(&self, s: &str) -> () {
        println!("STD ERROR: {}", s);
    }

    pub async fn std_out(&mut self, contract_args: Vec<ListParWithRandom>) -> () {
        if let Some((_, args)) = self.is_contract_call().unapply(contract_args) {
            if args.len() == 1 {
                // println!("\nhit std_out, arg: {:?}", args[0]);
                let str = self.pretty_printer.build_string_from_message(&args[0]);
                self.print_std_out(&str);
            } else {
                panic!("Expected exactly one argument, but got {}", args.len());
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub async fn std_out_ack(&mut self, contract_args: Vec<ListParWithRandom>) -> () {
        if let Some((produce, vec)) = self.is_contract_call().unapply(contract_args) {
            if let [arg, ack] = &vec[..] {
                // Debug print to verify the argument
                // println!("stdoutAck received arg: {:?}", arg);

                let str = self.pretty_printer.build_string_from_message(arg);
                self.print_std_out(&str);

                // Debug print before sending acknowledgment
                // println!("Sending acknowledgment on ack channel");

                if let Err(e) = produce(vec![Par::default()], ack.clone()).await {
                    eprintln!("Error producing result: {:?}", e);
                }
            } else {
                panic!("Expected exactly two arguments, but got {}", vec.len());
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub async fn std_err(&mut self, contract_args: Vec<ListParWithRandom>) -> () {
        if let Some((_, vec)) = self.is_contract_call().unapply(contract_args) {
            if let [arg] = &vec[..] {
                let str = self.pretty_printer.build_string_from_message(arg);
                self.print_std_err(&str);
            } else {
                panic!("Expected exactly one argument, but got {}", vec.len());
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub async fn std_err_ack(&mut self, contract_args: Vec<ListParWithRandom>) -> () {
        if let Some((produce, vec)) = self.is_contract_call().unapply(contract_args) {
            if let [arg, ack] = &vec[..] {
                let str = self.pretty_printer.build_string_from_message(arg);
                self.print_std_err(&str);
                if let Err(e) = produce(vec![Par::default()], ack.clone()).await {
                    eprintln!("Error producing result: {:?}", e);
                }
            } else {
                panic!("Expected exactly two arguments, but got {}", vec.len());
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub async fn rev_address(&self, contract_args: Vec<ListParWithRandom>) -> () {
        if let Some((produce, args)) = self.is_contract_call().unapply(contract_args) {
            match args.as_slice() {
                [first_par, second_par, ack] => {
                    if let (Some(_), Some(address)) = (
                        RhoString::unapply(first_par).and_then(|str| {
                            if str == "validate".to_string() {
                                Some(())
                            } else {
                                None
                            }
                        }),
                        RhoString::unapply(second_par),
                    ) {
                        let error_message = match RevAddress::parse(&address) {
                            Ok(_) => Par::default(),
                            Err(str) => RhoString::create_par(str),
                        };

                        if let Err(e) = produce(vec![error_message], ack.clone()).await {
                            eprintln!("Error producing result: {:?}", e);
                        }
                    } else {
                        // TODO: Invalid type for address should throw error! - OLD
                        if let Some(_) = RhoString::unapply(first_par).and_then(|str| {
                            if str == "validate".to_string() {
                                Some(())
                            } else {
                                None
                            }
                        }) {
                            if let Err(e) = produce(vec![Par::default()], ack.clone()).await {
                                eprintln!("Error producing result: {:?}", e);
                            }
                        } else {
                            if let (Some(_), Some(public_key)) = (
                                RhoString::unapply(first_par).and_then(|str| {
                                    if str == "fromPublicKey".to_string() {
                                        Some(())
                                    } else {
                                        None
                                    }
                                }),
                                RhoByteArray::unapply(second_par),
                            ) {
                                let response = match RevAddress::from_public_key(&PublicKey {
                                    bytes: public_key,
                                }) {
                                    Some(ra) => RhoString::create_par(ra.to_base58()),
                                    None => Par::default(),
                                };

                                if let Err(e) = produce(vec![response], ack.clone()).await {
                                    eprintln!("Error producing response: {:?}", e);
                                }
                            } else {
                                if let Some(_) = RhoString::unapply(first_par).and_then(|str| {
                                    if str == "fromPublicKey".to_string() {
                                        Some(())
                                    } else {
                                        None
                                    }
                                }) {
                                    if let Err(e) = produce(vec![Par::default()], ack.clone()).await
                                    {
                                        eprintln!("Error producing result: {:?}", e);
                                    }
                                } else {
                                    if let (Some(_), Some(id)) = (
                                        RhoString::unapply(first_par).and_then(|str| {
                                            if str == "fromDeployerId".to_string() {
                                                Some(())
                                            } else {
                                                None
                                            }
                                        }),
                                        RhoDeployerId::unapply(second_par),
                                    ) {
                                        let response = match RevAddress::from_deployer_id(id) {
                                            Some(ra) => RhoString::create_par(ra.to_base58()),
                                            None => Par::default(),
                                        };

                                        if let Err(e) = produce(vec![response], ack.clone()).await {
                                            eprintln!("Error producing response: {:?}", e);
                                        }
                                    } else {
                                        if let Some(_) =
                                            RhoString::unapply(first_par).and_then(|str| {
                                                if str == "fromDeployerId".to_string() {
                                                    Some(())
                                                } else {
                                                    None
                                                }
                                            })
                                        {
                                            if let Err(e) =
                                                produce(vec![Par::default()], ack.clone()).await
                                            {
                                                eprintln!("Error producing result: {:?}", e);
                                            }
                                        } else {
                                            if let Some(_) =
                                                RhoString::unapply(first_par).and_then(|str| {
                                                    if str == "fromUnforgeable".to_string() {
                                                        Some(())
                                                    } else {
                                                        None
                                                    }
                                                })
                                            {
                                                let response = if let Some(gprivate) =
                                                    RhoName::unapply(&second_par)
                                                {
                                                    RhoString::create_par(
                                                        RevAddress::from_unforgeable(&gprivate)
                                                            .to_base58(),
                                                    )
                                                } else {
                                                    Par::default()
                                                };

                                                if let Err(e) =
                                                    produce(vec![response], ack.clone()).await
                                                {
                                                    eprintln!("Error producing response: {:?}", e);
                                                }
                                            } else {
                                                if let Some(_) = RhoString::unapply(first_par)
                                                    .and_then(|str| {
                                                        if str == "fromUnforgeable".to_string() {
                                                            Some(())
                                                        } else {
                                                            None
                                                        }
                                                    })
                                                {
                                                    if let Err(e) =
                                                        produce(vec![Par::default()], ack.clone())
                                                            .await
                                                    {
                                                        eprintln!(
                                                            "Error producing result: {:?}",
                                                            e
                                                        );
                                                    }
                                                } else {
                                                    panic!("Expected args did not match any case");
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => panic!("args does not have exactly 3 elements"),
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub async fn deployer_id_ops(&self, contract_args: Vec<ListParWithRandom>) -> () {
        if let Some((produce, args)) = self.is_contract_call().unapply(contract_args) {
            match args.as_slice() {
                [first_par, second_par, ack] => {
                    if let (Some(_), Some(public_key)) = (
                        RhoString::unapply(first_par).and_then(|str| {
                            if str == "pubKeyBytes".to_string() {
                                Some(())
                            } else {
                                None
                            }
                        }),
                        RhoDeployerId::unapply(second_par),
                    ) {
                        if let Err(e) =
                            produce(vec![RhoByteArray::create_par(public_key)], ack.clone()).await
                        {
                            eprintln!("Error producing response: {:?}", e);
                        }
                    } else {
                        if let Some(_) = RhoString::unapply(first_par).and_then(|str| {
                            if str == "pubKeyBytes".to_string() {
                                Some(())
                            } else {
                                None
                            }
                        }) {
                            if let Err(e) = produce(vec![Par::default()], ack.clone()).await {
                                eprintln!("Error producing result: {:?}", e);
                            }
                        } else {
                            panic!("Expected args did not match any case");
                        }
                    }
                }

                _ => panic!("args does not have exactly 3 elements"),
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub async fn registry_ops(&self, contract_args: Vec<ListParWithRandom>) -> () {
        if let Some((produce, args)) = self.is_contract_call().unapply(contract_args) {
            match args.as_slice() {
                [first_par, argument, ack] => {
                    if let Some(_) = RhoString::unapply(first_par).and_then(|str| {
                        if str == "buildUri".to_string() {
                            Some(())
                        } else {
                            None
                        }
                    }) {
                        let response = if let Some(ba) = RhoByteArray::unapply(&argument) {
                            let hash_key_bytes = Blake2b256::hash(ba);
                            RhoUri::create_par(Registry::build_uri(&hash_key_bytes))
                        } else {
                            Par::default()
                        };
                        // println!("\nresponse in registry_ops: {:?}", response);
                        // println!("\nack in registry_ops: {:?}", ack);
                        if let Err(e) = produce(vec![response], ack.clone()).await {
                            eprintln!("Error producing response: {:?}", e);
                        }
                    } else {
                        panic!("Expected args did not match any case");
                    }
                }

                _ => panic!("args does not have exactly 3 elements"),
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub async fn sys_auth_token_ops(&self, contract_args: Vec<ListParWithRandom>) -> () {
        if let Some((produce, args)) = self.is_contract_call().unapply(contract_args) {
            match args.as_slice() {
                [first_par, argument, ack] => {
                    if let Some(_) = RhoString::unapply(first_par).and_then(|str| {
                        if str == "check".to_string() {
                            Some(())
                        } else {
                            None
                        }
                    }) {
                        let response: Expr = if let Some(_) = RhoByteArray::unapply(&argument) {
                            RhoBoolean::create_expr(true)
                        } else {
                            RhoBoolean::create_expr(false)
                        };

                        if let Err(e) =
                            produce(vec![Par::default().with_exprs(vec![response])], ack.clone())
                                .await
                        {
                            eprintln!("Error producing response: {:?}", e);
                        }
                    } else {
                        panic!("Expected args did not match any case");
                    }
                }

                _ => panic!("args does not have exactly 3 elements"),
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub async fn secp256k1_verify(&self, contract_args: Vec<ListParWithRandom>) -> () {
        self.verify_signature_contract(contract_args, "secp256k1Verify", Box::new(Secp256k1))
            .await
    }

    pub async fn ed25519_verify(&self, contract_args: Vec<ListParWithRandom>) -> () {
        self.verify_signature_contract(contract_args, "ed25519Verify", Box::new(Ed25519))
            .await
    }

    pub async fn sha256_hash(&self, contract_args: Vec<ListParWithRandom>) -> () {
        self.hash_contract(contract_args, "sha256Hash", Box::new(Sha256Hasher::hash))
            .await
    }

    pub async fn keccak256_hash(&self, contract_args: Vec<ListParWithRandom>) -> () {
        self.hash_contract(contract_args, "keccak256Hash", Box::new(Keccak256::hash))
            .await
    }

    pub async fn blake2b256_hash(&self, contract_args: Vec<ListParWithRandom>) -> () {
        self.hash_contract(contract_args, "blake2b256Hash", Box::new(Blake2b256::hash))
            .await
    }

    pub async fn get_block_data(
        &self,
        contract_args: Vec<ListParWithRandom>,
        block_data: Arc<RwLock<BlockData>>,
    ) -> () {
        if let Some((produce, args)) = self.is_contract_call().unapply(contract_args) {
            if args.len() == 1 {
                let data = block_data.read().unwrap();
                match produce(
                    vec![
                        Par::default().with_exprs(vec![RhoNumber::create_expr(data.block_number)]),
                        Par::default().with_exprs(vec![RhoNumber::create_expr(data.time_stamp)]),
                        RhoByteArray::create_par(data.sender.bytes.clone()),
                    ],
                    args[0].clone(),
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => eprintln!("Error producing block data: {:?}", e),
                };
            } else {
                panic!("blockData expects only a return channel");
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub async fn invalid_blocks(
        &self,
        contract_args: Vec<ListParWithRandom>,
        invalid_blocks: &InvalidBlocks,
    ) -> () {
        if let Some((produce, args)) = self.is_contract_call().unapply(contract_args) {
            if args.len() == 1 {
                let invalid_blocks: Par = invalid_blocks.invalid_blocks.read().unwrap().clone();

                if let Err(e) = produce(vec![invalid_blocks], args[0].clone()).await {
                    eprintln!("Error producing invalid blocks: {:?}", e);
                }
            } else {
                panic!("invalidBlocks expects only a return channel");
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    /*
     * The following functions below can be removed once rust-casper calls create_rho_runtime.
     * Until then, they must remain in the rholang directory to avoid circular dependencies.
     */

    // See casper/src/test/scala/coop/rchain/casper/helper/TestResultCollector.scala

    pub async fn handle_message(&self, message: Vec<ListParWithRandom>) -> () {
        // println!("\nhit handle_message");
        let mut printer = PrettyPrinter::new();

        fn clue_msg(clue: String, attempt: i64) -> String {
            format!("{} (test attempt: {})", clue, attempt)
        }

        if let Some((produce, assert_par)) = self.is_contract_call().unapply(message) {
            if let Some((test_name, attempt, assertion, clue, ack_channel)) =
                IsAssert::unapply(assert_par.clone())
            {
                if let Some((expected_or_unexpected, equals_or_not_equals_str, actual)) =
                    IsComparison::unapply(assertion.clone())
                {
                    if equals_or_not_equals_str == "==" {
                        let assertion = RhoTestAssertion::RhoAssertEquals {
                            test_name,
                            expected: expected_or_unexpected.clone(),
                            actual: actual.clone(),
                            clue: clue.clone(),
                        };

                        if let Err(e) = produce(
                            vec![new_gbool_par(assertion.is_success(), Vec::new(), false)],
                            ack_channel.clone(),
                        )
                        .await
                        {
                            eprintln!("Error producing result: {:?}", e);
                        }

                        assert_eq!(
                            printer.build_string_from_message(&actual),
                            printer.build_string_from_message(&expected_or_unexpected),
                            "{}",
                            clue_msg(clue, attempt)
                        );

                        assert_eq!(
                            actual,
                            expected_or_unexpected,
                            "{}",
                            clue_msg(clue, attempt)
                        );
                    } else if equals_or_not_equals_str == "!=" {
                        let assertion = RhoTestAssertion::RhoAssertNotEquals {
                            test_name,
                            unexpected: expected_or_unexpected.clone(),
                            actual: actual.clone(),
                            clue: clue.clone(),
                        };

                        if let Err(e) = produce(
                            vec![new_gbool_par(assertion.is_success(), Vec::new(), false)],
                            ack_channel.clone(),
                        )
                        .await
                        {
                            eprintln!("Error producing result: {:?}", e);
                        }

                        assert_ne!(
                            printer.build_string_from_message(&actual),
                            printer.build_string_from_message(&expected_or_unexpected),
                            "{}",
                            clue_msg(clue, attempt)
                        );

                        assert_ne!(
                            actual,
                            expected_or_unexpected,
                            "{}",
                            clue_msg(clue, attempt)
                        );
                    } else {
                        panic!("\nERROR: Expected args did not match any case");
                    }
                } else if let Some(condition) = RhoBoolean::unapply(&assertion) {
                    if let Err(e) = produce(
                        vec![new_gbool_par(condition, Vec::new(), false)],
                        ack_channel.clone(),
                    )
                    .await
                    {
                        eprintln!("Error producing result: {:?}", e);
                    }

                    assert_eq!(condition, true, "{}", clue_msg(clue, attempt));
                } else {
                    if let Err(e) = produce(
                        vec![new_gbool_par(false, Vec::new(), false)],
                        ack_channel.clone(),
                    )
                    .await
                    {
                        eprintln!("Error producing result: {:?}", e);
                    }

                    panic!("\nFailed to evaluate assertion: {:?}", assertion);
                }
            } else if let Some(_) = IsSetFinished::unapply(assert_par) {
                // println!("\nhas_finished: {}", has_finished);
            } else {
                panic!("\nERROR: Expected args did not match any case");
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    // See casper/src/test/scala/coop/rchain/casper/helper/RhoLoggerContract.scala

    pub async fn std_log(&mut self, message: Vec<ListParWithRandom>) -> () {
        if let Some((_, args)) = self.is_contract_call().unapply(message) {
            match args.as_slice() {
                [log_level_par, par] => {
                    if let Some(log_level) = RhoString::unapply(log_level_par) {
                        let msg = self.pretty_printer.build_string_from_message(par);

                        match log_level.as_str() {
                            "trace" => println!("trace: {}", msg),
                            "debug" => println!("debug: {}", msg),
                            "info" => println!("info: {}", msg),
                            "warn" => println!("warn: {}", msg),
                            "error" => println!("error: {}", msg),
                            _ => (),
                        }
                    } else {
                        ()
                    }
                }
                _ => (),
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    // See casper/src/test/scala/coop/rchain/casper/helper/DeployerIdContract.scala

    pub async fn deployer_id_make(&mut self, message: Vec<ListParWithRandom>) -> () {
        if let Some((produce, args)) = self.is_contract_call().unapply(message) {
            match args.as_slice() {
                [deployer_id_par, key_par, ack_channel] => {
                    if let (Some(deployer_id_str), Some(public_key)) = (
                        RhoString::unapply(deployer_id_par),
                        RhoByteArray::unapply(key_par),
                    ) {
                        if deployer_id_str == "deployerId" {
                            if let Err(e) = produce(
                                vec![RhoDeployerId::create_par(public_key.clone())],
                                ack_channel.clone(),
                            )
                            .await
                            {
                                eprintln!("Error producing DeployerId: {:?}", e);
                            }
                        } else {
                            panic!("Invalid deployerId call: {}", deployer_id_str);
                        }
                    } else {
                        panic!("Invalid arguments for deployerId:make");
                    }
                }
                _ => panic!("Invalid argument structure for deployerId:make"),
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    // See casper/src/test/scala/coop/rchain/casper/helper/Secp256k1SignContract.scala

    pub async fn secp256k1_sign(&mut self, message: Vec<ListParWithRandom>) -> () {
        if let Some((produce, args)) = self.is_contract_call().unapply(message) {
            match args.as_slice() {
                [hash_par, sk_par, ack_channel] => {
                    if let (Some(hash), Some(secret_key)) = (
                        RhoByteArray::unapply(hash_par),
                        RhoByteArray::unapply(sk_par),
                    ) {
                        if secret_key.len() != 32 {
                            panic!("Invalid private key length: must be 32 bytes");
                        }

                        let key_bytes = GenericArray::clone_from_slice(&secret_key);
                        let signing_key =
                            SigningKey::from_bytes(&key_bytes).expect("Invalid private key");

                        let signature: Signature = signing_key.sign(&hash);

                        let result_par = new_gbytearray_par(
                            signature.to_der().as_bytes().to_vec(),
                            Vec::new(),
                            false,
                        );

                        if let Err(e) = produce(vec![result_par], ack_channel.clone()).await {
                            eprintln!("Error producing Secp256k1 signature: {:?}", e);
                        }
                    } else {
                        panic!("Invalid arguments for secp256k1Sign");
                    }
                }
                _ => panic!("Invalid argument structure for secp256k1Sign"),
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    // See casper/src/test/scala/coop/rchain/casper/helper/SysAuthTokenContract.scala

    pub async fn sys_auth_token_make(&mut self, message: Vec<ListParWithRandom>) -> () {
        if let Some((produce, args)) = self.is_contract_call().unapply(message) {
            match args.as_slice() {
                [ack_channel] => {
                    let auth_token = new_gsys_auth_token_par(Vec::new(), false);

                    if let Err(e) = produce(vec![auth_token], ack_channel.clone()).await {
                        eprintln!("Error producing SysAuthToken: {:?}", e);
                    }
                }
                _ => panic!("Invalid argument structure for sys:test:authToken:make"),
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    //See casper/src/test/scala/coop/rchain/casper/helper/BlockDataContract.scala

    pub async fn block_data_set(&mut self, message: Vec<ListParWithRandom>) -> () {
        if let Some((produce, args)) = self.is_contract_call().unapply(message) {
            match args.as_slice() {
                [key_par, value_par, ack_channel] => {
                    if let Some(key) = RhoString::unapply(key_par) {
                        match key.as_str() {
                            "sender" => {
                                if let Some(public_key_bytes) = RhoByteArray::unapply(value_par) {
                                    let mut block_data = self.block_data.write().unwrap();
                                    block_data.sender = PublicKey {
                                        bytes: public_key_bytes.clone(),
                                    };

                                    let result_par = Par::default();
                                    if let Err(e) =
                                        produce(vec![result_par], ack_channel.clone()).await
                                    {
                                        eprintln!(
                                            "Error producing response for sender update: {:?}",
                                            e
                                        );
                                    }
                                } else {
                                    panic!("Invalid value for sender");
                                }
                            }
                            "blockNumber" => {
                                if let Some(block_number) = RhoNumber::unapply(value_par) {
                                    let mut block_data = self.block_data.write().unwrap();
                                    block_data.block_number = block_number;

                                    let result_par = Par::default();
                                    if let Err(e) =
                                        produce(vec![result_par], ack_channel.clone()).await
                                    {
                                        eprintln!(
                                            "Error producing response for blockNumber update: {:?}",
                                            e
                                        );
                                    }
                                } else {
                                    panic!("Invalid value for blockNumber");
                                }
                            }
                            _ => panic!("Unsupported key for block data update"),
                        }
                    } else {
                        panic!("Invalid key type for block data set");
                    }
                }
                _ => panic!("Invalid argument structure for block_data_set"),
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    // See casper/src/test/scala/coop/rchain/casper/helper/CasperInvalidBlocksContract.scala

    pub async fn casper_invalid_blocks_set(
        &self,
        message: Vec<ListParWithRandom>,
        invalid_blocks: &InvalidBlocks,
    ) -> () {
        if let Some((produce, args)) = self.is_contract_call().unapply(message) {
            match args.as_slice() {
                [new_invalid_blocks_par, ack_channel] => {
                    let mut invalid_blocks_lock = invalid_blocks.invalid_blocks.write().unwrap();
                    *invalid_blocks_lock = new_invalid_blocks_par.clone();

                    let result_par = Par::default();
                    if let Err(e) = produce(vec![result_par], ack_channel.clone()).await {
                        eprintln!(
                            "Error producing response for invalid blocks update: {:?}",
                            e
                        );
                    }
                }
                _ => panic!("Invalid argument structure for casper_invalid_blocks_set"),
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }
}

// See casper/src/test/scala/coop/rchain/casper/helper/RhoSpec.scala

pub fn test_framework_contracts() -> Vec<Definition> {
    vec![
        Definition {
            urn: "rho:test:assertAck".to_string(),
            fixed_channel: byte_name(101),
            arity: 5,
            body_ref: 101,
            handler: {
                Box::new(|ctx| {
                    Box::new(move |args| {
                        let ctx = ctx.clone();
                        Box::pin(
                            async move { ctx.system_processes.clone().handle_message(args).await },
                        )
                    })
                })
            },
            remainder: None,
        },
        Definition {
            urn: "rho:test:testSuiteCompleted".to_string(),
            fixed_channel: byte_name(102),
            arity: 1,
            body_ref: 102,
            handler: Box::new(|ctx| {
                Box::new(move |args| {
                    let ctx = ctx.clone();
                    Box::pin(async move { ctx.system_processes.clone().handle_message(args).await })
                })
            }),
            remainder: None,
        },
        Definition {
            urn: "rho:io:stdlog".to_string(),
            fixed_channel: byte_name(103),
            arity: 2,
            body_ref: 103,
            handler: Box::new(|ctx| {
                Box::new(move |args| {
                    let ctx = ctx.clone();
                    Box::pin(async move { ctx.system_processes.clone().std_log(args).await })
                })
            }),
            remainder: None,
        },
        Definition {
            urn: "rho:test:deployerId:make".to_string(),
            fixed_channel: byte_name(104),
            arity: 3,
            body_ref: 104,
            handler: Box::new(|ctx| {
                Box::new(move |args| {
                    let ctx = ctx.clone();
                    Box::pin(
                        async move { ctx.system_processes.clone().deployer_id_make(args).await },
                    )
                })
            }),
            remainder: None,
        },
        Definition {
            urn: "rho:test:crypto:secp256k1Sign".to_string(),
            fixed_channel: byte_name(105),
            arity: 3,
            body_ref: 105,
            handler: Box::new(|ctx| {
                Box::new(move |args| {
                    let ctx = ctx.clone();
                    Box::pin(async move { ctx.system_processes.clone().secp256k1_sign(args).await })
                })
            }),
            remainder: None,
        },
        Definition {
            urn: "sys:test:authToken:make".to_string(),
            fixed_channel: byte_name(106),
            arity: 1,
            body_ref: 106,
            handler: Box::new(|ctx| {
                Box::new(move |args| {
                    let ctx = ctx.clone();
                    Box::pin(
                        async move { ctx.system_processes.clone().sys_auth_token_make(args).await },
                    )
                })
            }),
            remainder: None,
        },
        Definition {
            urn: "rho:test:block:data:set".to_string(),
            fixed_channel: byte_name(107),
            arity: 3,
            body_ref: 107,
            handler: Box::new(|ctx| {
                Box::new(move |args| {
                    let ctx = ctx.clone();
                    Box::pin(async move { ctx.system_processes.clone().block_data_set(args).await })
                })
            }),
            remainder: None,
        },
        Definition {
            urn: "rho:test:casper:invalidBlocks:set".to_string(),
            fixed_channel: byte_name(108),
            arity: 2,
            body_ref: 108,
            handler: Box::new(|ctx| {
                let invalid_blocks = ctx.invalid_blocks.clone();
                Box::new(move |args| {
                    let invalid_blocks = invalid_blocks.clone();
                    let ctx = ctx.clone();
                    Box::pin(async move {
                        ctx.system_processes
                            .casper_invalid_blocks_set(args, &invalid_blocks)
                            .await
                    })
                })
            }),
            remainder: None,
        },
    ]
}

// See casper/src/test/scala/coop/rchain/casper/helper/TestResultCollector.scala

struct IsAssert;

impl IsAssert {
    pub fn unapply(p: Vec<Par>) -> Option<(String, i64, Par, String, Par)> {
        match p.as_slice() {
            [test_name_par, attempt_par, assertion_par, clue_par, ack_channel_par] => {
                if let (Some(test_name), Some(attempt), Some(clue)) = (
                    RhoString::unapply(test_name_par),
                    RhoNumber::unapply(attempt_par),
                    RhoString::unapply(clue_par),
                ) {
                    Some((
                        test_name,
                        attempt,
                        assertion_par.clone(),
                        clue,
                        ack_channel_par.clone(),
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

struct IsComparison;

impl IsComparison {
    pub fn unapply(p: Par) -> Option<(Par, String, Par)> {
        if let Some(expr) = single_expr(&p) {
            match expr.expr_instance.unwrap() {
                ExprInstance::ETupleBody(etuple) => match etuple.ps.as_slice() {
                    [expected_par, operator_par, actual_par] => {
                        if let Some(operator) = RhoString::unapply(operator_par) {
                            Some((expected_par.clone(), operator, actual_par.clone()))
                        } else {
                            None
                        }
                    }
                    _ => None,
                },

                _ => None,
            }
        } else {
            None
        }
    }
}

struct IsSetFinished;

impl IsSetFinished {
    pub fn unapply(p: Vec<Par>) -> Option<bool> {
        match p.as_slice() {
            [has_finished_par] => {
                if let Some(has_finished) = RhoBoolean::unapply(has_finished_par) {
                    Some(has_finished)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum RhoTestAssertion {
    RhoAssertTrue {
        test_name: String,
        is_success: bool,
        clue: String,
    },

    RhoAssertEquals {
        test_name: String,
        expected: Par,
        actual: Par,
        clue: String,
    },

    RhoAssertNotEquals {
        test_name: String,
        unexpected: Par,
        actual: Par,
        clue: String,
    },
}

impl RhoTestAssertion {
    pub fn is_success(&self) -> bool {
        match self {
            RhoTestAssertion::RhoAssertTrue { is_success, .. } => *is_success,
            RhoTestAssertion::RhoAssertEquals {
                expected, actual, ..
            } => actual == expected,
            RhoTestAssertion::RhoAssertNotEquals {
                unexpected, actual, ..
            } => actual != unexpected,
        }
    }
}
