// See crypto/src/main/scala/coop/rchain/crypto/hash/Blake2b512Random.scala
/** Blake2b512 based splittable and mergeable random number generator
 * specialized for generating 256-bit unforgeable names.
 * splitByte and splitShort are the interfaces to make the random number
 * generator diverge.
 * Blake2b512.merge uses online tree hashing to merge two random generator
 * states.
 *
 * TODO: This might be turned into a crate
 * TODO: Investigate this as the custom type in RhoTypes.proto
 */
pub struct Blake2b512Random {
    pub bytes: Vec<u8>,
}
