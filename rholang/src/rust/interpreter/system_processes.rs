use crypto::rust::hash::blake2b256::Blake2b256;
use crypto::rust::hash::keccak256::Keccak256;
use crypto::rust::hash::sha_256::Sha256Hasher;
use crypto::rust::public_key::PublicKey;
use crypto::rust::signatures::ed25519::Ed25519;
use crypto::rust::signatures::secp256k1::Secp256k1;
use crypto::rust::signatures::signatures_alg::SignaturesAlg;
use models::rhoapi::g_unforgeable::UnfInstance::GPrivateBody;
use models::rhoapi::Expr;
use models::rhoapi::{Bundle, GPrivate, GUnforgeable, ListParWithRandom, Par, Var};
use models::rust::casper::protocol::casper_message::BlockMessage;
use models::Byte;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

use super::contract_call::ContractCall;
use super::dispatch::RhoDispatch;
use super::pretty_printer::PrettyPrinter;
use super::registry::registry::Registry;
use super::rho_runtime::RhoTuplespace;
use super::rho_type::{
    RhoBoolean, RhoByteArray, RhoDeployerId, RhoName, RhoNumber, RhoString, RhoUri,
};
use super::util::rev_address::RevAddress;

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/SystemProcesses.scala
// NOTE: Not implementing Logger
pub type RhoSysFunction = Box<dyn FnMut(Vec<ListParWithRandom>) -> ()>;
pub type RhoDispatchMap = HashMap<i64, RhoSysFunction>;
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

fn byte_name(b: Byte) -> Par {
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

pub struct ProcessContext {
    pub space: RhoTuplespace,
    pub dispatcher: RhoDispatch,
    pub block_data: Arc<RwLock<BlockData>>,
    pub invalid_blocks: InvalidBlocks,
    pub system_processes: SystemProcesses,
}

impl ProcessContext {
    pub fn create(
        space: RhoTuplespace,
        dispatcher: RhoDispatch,
        block_data: Arc<RwLock<BlockData>>,
        invalid_blocks: InvalidBlocks,
    ) -> Self {
        ProcessContext {
            space: space.clone(),
            dispatcher: dispatcher.clone(),
            block_data,
            invalid_blocks,
            system_processes: SystemProcesses::create(dispatcher, space),
        }
    }
}

pub struct Definition {
    pub urn: String,
    pub fixed_channel: Name,
    pub arity: Arity,
    pub body_ref: BodyRef,
    pub handler: Box<dyn FnMut(ProcessContext) -> Box<dyn FnMut(Vec<ListParWithRandom>) -> ()>>,
    pub remainder: Remainder,
}

impl Definition {
    pub fn new(
        urn: String,
        fixed_channel: Name,
        arity: Arity,
        body_ref: BodyRef,
        handler: Box<dyn FnMut(ProcessContext) -> Box<dyn FnMut(Vec<ListParWithRandom>) -> ()>>,
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
    ) -> (BodyRef, Box<dyn FnMut(Vec<ListParWithRandom>) -> ()>) {
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
    time_stamp: i64,
    block_number: i64,
    sender: PublicKey,
    seq_num: i32,
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

pub struct SystemProcesses {
    pub dispatcher: RhoDispatch,
    pub space: RhoTuplespace,
    pretty_printer: PrettyPrinter,
}

impl SystemProcesses {
    fn create(dispatcher: RhoDispatch, space: RhoTuplespace) -> Self {
        SystemProcesses {
            dispatcher,
            space,
            pretty_printer: PrettyPrinter::new(),
        }
    }

    fn is_contract_call(&self) -> ContractCall {
        ContractCall {
            space: self.space.clone(),
            dispatcher: self.dispatcher.clone(),
        }
    }

    fn verify_signature_contract(
        &self,
        contract_args: Vec<ListParWithRandom>,
        name: &str,
        // algorithm: Box<dyn Fn(Vec<u8>, Vec<u8>, Vec<u8>) -> bool>,
        algorithm: Box<dyn SignaturesAlg>,
    ) -> () {
        if let Some((produce, vec)) = self.is_contract_call().unapply(contract_args) {
            if let [data, signature, pub_key, ack] = &vec[..] {
                if let (Some(data_bytes), Some(signature_bytes), Some(pub_key_bytes)) = (
                    RhoByteArray::unapply(&data),
                    RhoByteArray::unapply(&signature),
                    RhoByteArray::unapply(&pub_key),
                ) {
                    let verified = algorithm.verify(&data_bytes, &signature_bytes, pub_key_bytes);
                    if !verified {
                        panic!("SystemProcesses: Failed to verify contract")
                    }
                    let _ = produce(
                        vec![Par::default().with_exprs(vec![RhoBoolean::create_expr(verified)])],
                        ack.clone(),
                    );
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

    pub fn hash_contract(
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
                    let _ = produce(vec![RhoByteArray::create_par(hash)], ack.clone());
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

    pub fn std_out(&mut self, contract_args: Vec<ListParWithRandom>) -> () {
        if let Some((_, args)) = self.is_contract_call().unapply(contract_args) {
            if args.len() == 1 {
                let str = self.pretty_printer.build_string_from_message(&args[0]);
                self.print_std_out(&str);
            } else {
                panic!("Expected exactly one argument, but got {}", args.len());
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub fn std_out_ack(&mut self, contract_args: Vec<ListParWithRandom>) -> () {
        if let Some((produce, vec)) = self.is_contract_call().unapply(contract_args) {
            if let [arg, ack] = &vec[..] {
                let str = self.pretty_printer.build_string_from_message(arg);
                self.print_std_out(&str);
                let _ = produce(vec![Par::default()], ack.clone());
            } else {
                panic!("Expected exactly two arguments, but got {}", vec.len());
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub fn std_err(&mut self, contract_args: Vec<ListParWithRandom>) -> () {
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

    pub fn std_err_ack(&mut self, contract_args: Vec<ListParWithRandom>) -> () {
        if let Some((produce, vec)) = self.is_contract_call().unapply(contract_args) {
            if let [arg, ack] = &vec[..] {
                let str = self.pretty_printer.build_string_from_message(arg);
                self.print_std_err(&str);
                let _ = produce(vec![Par::default()], ack.clone());
            } else {
                panic!("Expected exactly two arguments, but got {}", vec.len());
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub fn rev_address(&self, contract_args: Vec<ListParWithRandom>) -> () {
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

                        let _ = produce(vec![error_message], ack.clone());
                    } else {
                        // TODO: Invalid type for address should throw error!
                        if let Some(_) = RhoString::unapply(first_par).and_then(|str| {
                            if str == "validate".to_string() {
                                Some(())
                            } else {
                                None
                            }
                        }) {
                            let _ = produce(vec![Par::default()], ack.clone());
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

                                let _ = produce(vec![response], ack.clone());
                            } else {
                                if let Some(_) = RhoString::unapply(first_par).and_then(|str| {
                                    if str == "fromPublicKey".to_string() {
                                        Some(())
                                    } else {
                                        None
                                    }
                                }) {
                                    let _ = produce(vec![Par::default()], ack.clone());
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

                                        let _ = produce(vec![response], ack.clone());
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
                                            let _ = produce(vec![Par::default()], ack.clone());
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

                                                let _ = produce(vec![response], ack.clone());
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
                                                    let _ =
                                                        produce(vec![Par::default()], ack.clone());
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

    pub fn deployer_id_ops(&self, contract_args: Vec<ListParWithRandom>) -> () {
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
                        let _ = produce(vec![RhoByteArray::create_par(public_key)], ack.clone());
                    } else {
                        if let Some(_) = RhoString::unapply(first_par).and_then(|str| {
                            if str == "pubKeyBytes".to_string() {
                                Some(())
                            } else {
                                None
                            }
                        }) {
                            let _ = produce(vec![Par::default()], ack.clone());
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

    pub fn registry_ops(&self, contract_args: Vec<ListParWithRandom>) -> () {
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
                        let _ = produce(vec![response], ack.clone());
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

    pub fn sys_auth_token_ops(&self, contract_args: Vec<ListParWithRandom>) -> () {
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
                        let _ =
                            produce(vec![Par::default().with_exprs(vec![response])], ack.clone());
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

    pub fn secp256k1_verify(&self, contract_args: Vec<ListParWithRandom>) -> () {
        self.verify_signature_contract(contract_args, "secp256k1Verify", Box::new(Secp256k1))
    }

    pub fn ed25519_verify(&self, contract_args: Vec<ListParWithRandom>) -> () {
        self.verify_signature_contract(contract_args, "ed25519Verify", Box::new(Ed25519))
    }

    pub fn sha256_hash(&self, contract_args: Vec<ListParWithRandom>) -> () {
        self.hash_contract(contract_args, "sha256Hash", Box::new(Sha256Hasher::hash))
    }

    pub fn keccak256_hash(&self, contract_args: Vec<ListParWithRandom>) -> () {
        self.hash_contract(contract_args, "keccak256Hash", Box::new(Keccak256::hash))
    }

    pub fn blake2b256_hash(&self, contract_args: Vec<ListParWithRandom>) -> () {
        self.hash_contract(contract_args, "blake2b256Hash", Box::new(Blake2b256::hash))
    }

    pub fn get_block_data(
        &self,
        contract_args: Vec<ListParWithRandom>,
        block_data: Arc<RwLock<BlockData>>,
    ) -> () {
        if let Some((produce, args)) = self.is_contract_call().unapply(contract_args) {
            if args.len() == 1 {
                let data = block_data.read().unwrap();
                let _ = produce(
                    vec![
                        Par::default().with_exprs(vec![RhoNumber::create_expr(data.block_number)]),
                        Par::default().with_exprs(vec![RhoNumber::create_expr(data.time_stamp)]),
                        RhoByteArray::create_par(data.sender.bytes.clone()),
                    ],
                    args[0].clone(),
                );
            } else {
                panic!("blockData expects only a return channel");
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub fn invalid_blocks(
        &self,
        contract_args: Vec<ListParWithRandom>,
        invalid_blocks: &InvalidBlocks,
    ) -> () {
        if let Some((produce, args)) = self.is_contract_call().unapply(contract_args) {
            if args.len() == 1 {
                let invalid_blocks: Par = invalid_blocks.invalid_blocks.read().unwrap().clone();

                let _ = produce(vec![invalid_blocks], args[0].clone());
            } else {
                panic!("invalidBlocks expects only a return channel");
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }
}
