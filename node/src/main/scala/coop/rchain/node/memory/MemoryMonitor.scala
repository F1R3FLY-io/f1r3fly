package coop.rchain.node.memory

import cats.effect.{Concurrent, Sync, Timer}
import cats.syntax.all._
import coop.rchain.rholang.JNAInterfaceLoader.RHOLANG_RUST_INSTANCE
import coop.rchain.shared.Log
import rspacePlusPlus.JNAInterfaceLoader.{INSTANCE => RSPACE_INSTANCE}

import scala.concurrent.duration._

/**
  * Memory monitoring service that tracks native memory allocation from Rust libraries
  */
object MemoryMonitor {

  case class MemoryStats(
      rholangAllocated: Long,
      rspaceAllocated: Long,
      totalAllocated: Long,
      jvmHeapUsed: Long,
      jvmHeapMax: Long
  ) {
    def prettyPrint: String =
      s"""Memory Stats:
         |  Native Rholang: ${MemoryMonitor.formatBytes(rholangAllocated)}
         |  Native RSpace:  ${MemoryMonitor.formatBytes(rspaceAllocated)}
         |  Native Total:   ${MemoryMonitor.formatBytes(totalAllocated)}
         |  JVM Heap Used:  ${MemoryMonitor.formatBytes(jvmHeapUsed)}
         |  JVM Heap Max:   ${MemoryMonitor.formatBytes(jvmHeapMax)}""".stripMargin
  }

  private def formatBytes(bytes: Long): String =
    if (bytes == 0) "0 B"
    else if (bytes < 1024) s"$bytes B"
    else if (bytes < 1024 * 1024) s"${bytes / 1024} KB"
    else if (bytes < 1024 * 1024 * 1024) s"${bytes / (1024 * 1024)} MB"
    else s"${bytes / (1024 * 1024 * 1024)} GB"

  /**
    * Get current memory statistics
    */
  def getMemoryStats: MemoryStats = {
    val runtime          = Runtime.getRuntime
    val rholangAllocated = RHOLANG_RUST_INSTANCE.rholang_get_allocated_bytes()
    val rspaceAllocated  = RSPACE_INSTANCE.get_allocated_bytes()

    MemoryStats(
      rholangAllocated = rholangAllocated,
      rspaceAllocated = rspaceAllocated,
      totalAllocated = rholangAllocated + rspaceAllocated,
      jvmHeapUsed = runtime.totalMemory() - runtime.freeMemory(),
      jvmHeapMax = runtime.maxMemory()
    )
  }

  /**
    * Reset native memory counters (useful for testing)
    */
  def resetCounters(): Unit = {
    RHOLANG_RUST_INSTANCE.rholang_reset_allocated_bytes()
    RSPACE_INSTANCE.reset_allocated_bytes()
  }

  /**
    * Check for memory leaks and log warnings if found
    */
  def checkForLeaks[F[_]: Sync: Log]: F[Unit] =
    for {
      stats <- Sync[F].delay(getMemoryStats)
      _ <- if (stats.rholangAllocated > 0) {
            Log[F].warn(
              s"🔴 RHOLANG MEMORY LEAK: ${formatBytes(stats.rholangAllocated)} not deallocated!"
            )
          } else if (stats.rspaceAllocated > 0) {
            Log[F].warn(
              s"🔴 RSPACE MEMORY LEAK: ${formatBytes(stats.rspaceAllocated)} not deallocated!"
            )
          } else {
            Log[F].debug("🟢 No native memory leaks detected - all clean!")
          }
    } yield ()

  /**
    * Monitor memory around a specific operation
    */
  def monitorOperation[F[_]: Sync: Log, A](operationName: String)(operation: F[A]): F[A] =
    for {
      startStats   <- Sync[F].delay(getMemoryStats)
      result       <- operation
      endStats     <- Sync[F].delay(getMemoryStats)
      nativeGrowth = endStats.totalAllocated - startStats.totalAllocated
      heapGrowth   = endStats.jvmHeapUsed - startStats.jvmHeapUsed
      _ <- if (nativeGrowth > 0) {
            Log[F].warn(
              s"⚠️  MEMORY LEAK DETECTED in '$operationName': leaked ${formatBytes(nativeGrowth)} of native memory"
            )
          } else if (heapGrowth > 1024 * 1024) { // 1MB threshold
            Log[F].info(
              s"📊 Operation '$operationName' used ${formatBytes(heapGrowth)} of heap memory"
            )
          } else if (nativeGrowth < 0) {
            Log[F].debug(
              s"✅ Operation '$operationName' freed ${formatBytes(-nativeGrowth)} of native memory"
            )
          } else {
            Log[F].debug(s"✅ Operation '$operationName' completed with no memory growth")
          }
    } yield result

  /**
    * Start periodic memory monitoring
    */
  def startPeriodicMonitoring[F[_]: Concurrent: Timer: Log](
      interval: FiniteDuration = 30.seconds,
      warnThreshold: Long = 100 * 1024 * 1024 // 100MB
  ): F[Unit] = {

    def checkAndLog: F[Unit] =
      for {
        stats <- Sync[F].delay(getMemoryStats)
        _ <- if (stats.totalAllocated > warnThreshold) {
              Log[F].warn(s"🚨 HIGH NATIVE MEMORY USAGE DETECTED:\n${stats.prettyPrint}")
            } else if (stats.totalAllocated > 0) {
              Log[F].info(s"📈 Native memory in use:\n${stats.prettyPrint}")
            } else {
              Log[F].debug(s"💚 Memory status clean:\n${stats.prettyPrint}")
            }
      } yield ()

    def periodicLoop: F[Unit] =
      for {
        _ <- Timer[F].sleep(interval)
        _ <- checkAndLog
        _ <- periodicLoop
      } yield ()

    for {
      _ <- Log[F].info(
            s"🔍 Starting comprehensive memory monitoring every $interval (threshold: ${formatBytes(warnThreshold)})"
          )
      _ <- checkAndLog
      _ <- periodicLoop
    } yield ()
  }
}
