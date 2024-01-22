package fs2chat

import scala.concurrent.duration._

import cats.effect.Concurrent
import fs2.concurrent.Queue
import cats.implicits._
import fs2.Stream
import fs2.io.tcp.Socket
import scodec.stream.{StreamDecoder, StreamEncoder}
import scodec.{Decoder, Encoder}

/** Socket which reads a stream of messages of type `In` and allows writing messages of type `Out`.
  */
trait MessageSocket[F[_], In, Out] {
  def read: Stream[F, In]
  def write1(out: Out): F[Unit]
}

object MessageSocket {
  val PACKET_SIZE = 1024

  def apply[F[_]: Concurrent, In, Out](
      socket: Socket[F],
      inDecoder: Decoder[In],
      outEncoder: Encoder[Out],
      outputBound: Int
  ): F[MessageSocket[F, In, Out]] =
    for {
      outgoing <- Queue.bounded[F, Out](outputBound)
    } yield new MessageSocket[F, In, Out] {
      def read: Stream[F, In] = {
        val readSocket = socket
          .reads(PACKET_SIZE, none[FiniteDuration])
          .through(StreamDecoder.many(inDecoder).toPipeByte[F])

        val writeOutput = outgoing.dequeue
          .through(StreamEncoder.many(outEncoder).toPipeByte)
          .through(socket.writes(none[FiniteDuration]))

        readSocket.concurrently(writeOutput)
      }
      def write1(out: Out): F[Unit] = outgoing.enqueue1(out)
    }
}
