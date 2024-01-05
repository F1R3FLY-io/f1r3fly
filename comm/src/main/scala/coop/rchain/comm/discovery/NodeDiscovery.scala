package coop.rchain.comm.discovery

import cats.tagless._
import cats.Monad
import coop.rchain.comm.{NodeIdentifier, PeerNode}

import coop.rchain.shared.Log

@autoFunctorK
@autoSemigroupalK
@autoProductNK
trait NodeDiscovery[F[_]] {
  def discover: F[Unit]
  def peers: F[Seq[PeerNode]]
}

object NodeDiscovery extends NodeDiscoveryInstances {
  def apply[F[_]](implicit L: NodeDiscovery[F]): NodeDiscovery[F] = L
}

sealed abstract class NodeDiscoveryInstances {

  def kademlia[F[_]: Monad: KademliaStore: KademliaRPC: Log](id: NodeIdentifier): NodeDiscovery[F] =
    new NodeDiscovery[F] {
      def discover: F[Unit]       = KademliaNodeDiscovery.discover(id)
      def peers: F[Seq[PeerNode]] = KademliaNodeDiscovery.peers
    }
}
