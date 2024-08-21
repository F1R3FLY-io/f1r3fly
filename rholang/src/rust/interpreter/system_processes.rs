use std::sync::{Arc, Mutex, RwLock};

use crypto::rust::public_key::PublicKey;
use models::rhoapi::g_unforgeable::UnfInstance::GPrivateBody;
use models::rhoapi::{Bundle, GPrivate, GUnforgeable, ListParWithRandom, Par, Var};
use models::rust::casper::protocol::casper_message::BlockMessage;
use models::Byte;

use super::contract_call::ContractCall;
use super::dispatch::{RhoDispatch, RholangAndRustDispatcher};
use super::pretty_printer::PrettyPrinter;
use super::rho_runtime::RhoTuplespace;
use super::rho_type::{Boolean, ByteArray};

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/SystemProcesses.scala
// NOTE: Not implementing Logger
pub type Name = Par;
pub type Arity = i32;
pub type Remainder = Option<Var>;
pub type BodyRef = i64;
pub type Contract = dyn Fn(Vec<ListParWithRandom>) -> ();

pub struct InvalidBlocks {
    invalid_blocks: Arc<RwLock<Par>>,
}
impl InvalidBlocks {
    pub fn set_params(&self, invalid_blocks: Par) -> () {
        let mut lock = self.invalid_blocks.write().unwrap();

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
    space: RhoTuplespace,
    dispatcher: RhoDispatch,
    block_data: Arc<RwLock<BlockData>>,
    invalid_blocks: InvalidBlocks,
    system_processes: SystemProcesses,
}

pub struct Definition {
    urn: String,
    fixed_channel: Name,
    arity: Arity,
    body_ref: BodyRef,
    handler: fn(&ProcessContext) -> fn(Vec<ListParWithRandom>) -> (),
    remainder: Remainder,
}

impl Definition {
    pub fn new(
        urn: String,
        fixed_channel: Name,
        arity: Arity,
        body_ref: BodyRef,
        handler: fn(&ProcessContext) -> fn(Vec<ListParWithRandom>) -> (),
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
        &self,
        context: &ProcessContext,
    ) -> (BodyRef, fn(Vec<ListParWithRandom>) -> ()) {
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
    dispatcher: Arc<Mutex<RholangAndRustDispatcher>>,
    space: RhoTuplespace,
    pretty_printer: PrettyPrinter,
}

impl SystemProcesses {
    fn is_contract_call(&self) -> ContractCall {
        ContractCall {
            space: self.space.clone(),
            dispatcher: self.dispatcher.clone(),
        }
    }

    fn illegal_argument_exception(&self, msg: &str) -> () {
        panic!("{}", msg);
    }

    fn verify_signature_contract(
        &self,
        contract_args: Vec<ListParWithRandom>,
        name: &str,
        algorithm: Box<dyn Fn(Vec<u8>, Vec<u8>, Vec<u8>) -> bool>,
    ) -> () {
        if let Some((produce, vec)) = self.is_contract_call().unapply(contract_args) {
            if let [data, signature, pub_key, ack] = &vec[..] {
                if let (Some(data_bytes), Some(signature_bytes), Some(pub_key_bytes)) = (
                    ByteArray::unapply(&data),
                    ByteArray::unapply(&signature),
                    ByteArray::unapply(&pub_key),
                ) {
                    let verified = algorithm(data_bytes, signature_bytes, pub_key_bytes);
                    let _ = produce(
                        vec![Par::default().with_exprs(vec![Boolean::create_expr(verified)])],
                        ack.clone(),
                    );

                    ()
                } else {
                    panic!("{} expects data, signature, public key (all as byte arrays), and an acknowledgement channel", name)
                }
            } else {
                panic!("Invalid contract call")
            }
        } else {
            panic!("Invalid contract call")
        }
    }

    fn print_std_out(&self, s: &str) -> () {
        println!("{}", s);
    }

    pub fn std_out(&self, contract_args: Vec<ListParWithRandom>) -> () {
        if let Some((_, args)) = self.is_contract_call().unapply(contract_args) {
            if args.len() == 1 {
                self.print_std_out(&PrettyPrinter::build_string_from_par(args[0].clone()));
            } else {
                panic!("Expected exactly one argument, but got {}", args.len());
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }

    pub fn std_out_ack() -> () {
        todo!()
    }

    pub fn std_err() -> () {
        todo!()
    }

    pub fn std_err_ack() -> () {
        todo!()
    }

    pub fn secp256k1_verify() -> () {
        todo!()
    }

    pub fn ed25519_verify() -> () {
        todo!()
    }

    pub fn sha256_hash() -> () {
        todo!()
    }

    pub fn keccak256_hash() -> () {
        todo!()
    }

    pub fn blake2b256_hash() -> () {
        todo!()
    }

    pub fn get_block_data(block_data: &BlockData) -> () {
        todo!()
    }

    pub fn invalid_blocks(invalid_blocks: &InvalidBlocks) -> () {
        todo!()
    }

    pub fn rev_address() -> () {
        todo!()
    }

    pub fn deployer_id_ops() -> () {
        todo!()
    }

    pub fn registry_ops() -> () {
        todo!()
    }

    pub fn sys_auth_token_ops() -> () {
        todo!()
    }
}
