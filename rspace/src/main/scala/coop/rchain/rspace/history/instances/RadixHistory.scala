package coop.rchain.rspace.history.instances

import cats.Parallel
import cats.effect.Sync
import cats.syntax.all._
import coop.rchain.rspace.hashing.Blake2b256Hash
import coop.rchain.rspace.history.RadixTree._
import coop.rchain.rspace.history._
import coop.rchain.shared.syntax.sharedSyntaxKeyValueStore
import coop.rchain.store.{KeyValueStore, KeyValueTypedStore}
import scodec.bits.ByteVector

/**
  * History implementation with radix tree
  */
object RadixHistory {
  val emptyRootHash: Blake2b256Hash = Blake2b256Hash.fromByteVector(hashNode(emptyNode)._1)

  def apply[F[_]: Sync: Parallel](
      root: Blake2b256Hash,
      store: KeyValueTypedStore[F, ByteVector, ByteVector]
  ): F[RadixHistory[F]] =
    for {
      impl <- Sync[F].delay(new RadixTreeImpl[F](store))
      node <- impl.loadNode(root.bytes, noAssert = true)
      // _ = println(
      //   "\nRadix Tree after Radix History save_node: "
      // )
      // _ <- impl
      //       .printTree(node, "", false)
    } yield RadixHistory(root, node, impl, store)

  def createStore[F[_]: Sync](
      store: KeyValueStore[F]
  ): KeyValueTypedStore[F, ByteVector, ByteVector] =
    store.toTypedStore(scodec.codecs.bytes, scodec.codecs.bytes)
}

final case class RadixHistory[F[_]: Sync: Parallel](
    rootHash: Blake2b256Hash,
    rootNode: Node,
    impl: RadixTreeImpl[F],
    store: KeyValueTypedStore[F, ByteVector, ByteVector]
) extends History[F] {
  override type HistoryF = History[F]

  override def root: Blake2b256Hash = rootHash

  override def reset(root: Blake2b256Hash): F[History[F]] =
    for {
      impl <- Sync[F].delay(new RadixTreeImpl[F](store))
      node <- impl.loadNode(root.bytes, noAssert = true)
      // _    = println("\nrootNode in reset: " + node)
    } yield this.copy(root, node, impl, store)

  override def read(key: ByteVector): F[Option[ByteVector]] =
    impl.read(rootNode, key)

  override def process(actions: List[HistoryAction]): F[History[F]] =
    for {

      /** TODO: To improve time, it is possible to implement this check into the [[RadixTreeImpl.makeActions()]]. */
      _ <- new RuntimeException("Cannot process duplicate actions on one key.").raiseError
            .unlessA(hasNoDuplicates(actions))

      // _ = println("\nhit process")

      // _              = println("\nroot_node (curr_node) in process method: " + rootNode)
      newRootNodeOpt <- impl.makeActions(rootNode, actions)
      // _              = println("\nnewRootNodeOpt: " + newRootNodeOpt)
      newHistoryOpt <- newRootNodeOpt.traverse { newRootNode =>
                        // println("\nnewRootNode in process: " + newRootNode)
                        // val hash = impl.saveNode(newRootNode)
                        // // println("\nRadix Tree after Radix History save_node: ")
                        // for {
                        //   // _                <- impl.printTree(newRootNode, "", false)
                        //   blakeHash        = Blake2b256Hash.fromByteVector(hash)
                        //   newHistory       = this.copy(blakeHash, newRootNode, impl, store)
                        //   committedHistory <- impl.commit.as(newHistory)
                        //   // store_map        <- store.toMap
                        //   // _                = println("\nstore after commit: " + store_map)
                        // } yield committedHistory

                        val hash      = impl.saveNode(newRootNode)
                        val blakeHash = Blake2b256Hash.fromByteVector(hash)
                        // println("\nnewRootNode hash: " + blakeHash.bytes)
                        val newHistory = this.copy(blakeHash, newRootNode, impl, store)
                        impl.commit.as(newHistory)
                      }
      _ = impl.clearWriteCache()
      _ = impl.clearReadCache()
      // _ = println("\nPrinting Radix Tree after process: ")
      // _ <- impl.printTree(this.rootNode, "", false)
    } yield newHistoryOpt.getOrElse(this)

  private def hasNoDuplicates(actions: List[HistoryAction]) =
    actions.map(_.key).distinct.size == actions.size
}
