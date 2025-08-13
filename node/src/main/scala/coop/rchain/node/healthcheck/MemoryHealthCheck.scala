package coop.rchain.node.healthcheck

import cats.effect.Sync
import cats.syntax.all._
import coop.rchain.node.memory.MemoryMonitor
import io.circe.syntax._
import io.circe.{Encoder, Json}

object MemoryHealthCheck {

  implicit val memoryStatsEncoder: Encoder[MemoryMonitor.MemoryStats] = Encoder.instance { stats =>
    Json.obj(
      "native_rholang_allocated" -> stats.rholangAllocated.asJson,
      "native_rspace_allocated"  -> stats.rspaceAllocated.asJson,
      "native_total_allocated"   -> stats.totalAllocated.asJson,
      "jvm_heap_used"            -> stats.jvmHeapUsed.asJson,
      "jvm_heap_max"             -> stats.jvmHeapMax.asJson,
      "memory_status"            -> (if (stats.totalAllocated == 0) "healthy" else "leaking").asJson
    )
  }

  /**
    * Get memory status as JSON for health check endpoints
    */
  def getMemoryStatus[F[_]: Sync]: F[Json] =
    Sync[F].delay {
      val stats = MemoryMonitor.getMemoryStats
      stats.asJson
    }

  /**
    * Simple health check that returns success/failure based on memory state
    */
  def healthCheck[F[_]: Sync]: F[Boolean] =
    Sync[F].delay {
      val stats = MemoryMonitor.getMemoryStats
      // Consider healthy if native memory is under 500MB and no obvious leaks
      stats.totalAllocated < 500 * 1024 * 1024
    }
}
