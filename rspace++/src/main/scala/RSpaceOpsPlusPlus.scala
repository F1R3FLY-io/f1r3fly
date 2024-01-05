package rspacePlusPlus

import cats.effect.Concurrent
import coop.rchain.shared.Log

abstract class RSpaceOpsPlusPlus[F[_]: Concurrent: Log]() {}
