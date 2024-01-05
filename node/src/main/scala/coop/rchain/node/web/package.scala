package coop.rchain.node

import cats.effect.{ConcurrentEffect, ExitCode, Sync, Timer}
import cats.syntax.all._
import coop.rchain.comm.discovery.NodeDiscovery
import coop.rchain.comm.rp.Connect.{ConnectionsCell, RPConfAsk}
import coop.rchain.node.api.{AdminWebApi, WebApi}
import coop.rchain.node.diagnostics.NewPrometheusReporter
import coop.rchain.node.effects.EventConsumer
import coop.rchain.node.web.ReportingRoutes.ReportingHttpRoutes
import coop.rchain.node.web.https4s.RouterFix
import coop.rchain.shared.Log
import monix.execution.Scheduler
import org.http4s.HttpRoutes
import org.http4s.implicits._
import org.http4s.server.blaze.BlazeServerBuilder
import org.http4s.server.middleware.CORS

import scala.concurrent.duration.{DurationInt, FiniteDuration}

package object web {
  // https://github.com/http4s/http4s/security/advisories/GHSA-52cf-226f-rhr6
  //  val corsPolicy = CORS.policy.withAllowCredentials(false) // after http4s v0.22.x
  def corsPolicy[F[_]: Sync](routes: HttpRoutes[F]) =
    CORS(routes, CORS.DefaultCORSConfig.copy(allowCredentials = false))

  def aquireHttpServer[F[_]: ConcurrentEffect: Timer: RPConfAsk: NodeDiscovery: ConnectionsCell: EventConsumer: Log](
      reporting: Boolean,
      host: String = "0.0.0.0",
      httpPort: Int,
      prometheusReporter: NewPrometheusReporter,
      connectionIdleTimeout: FiniteDuration,
      webApi: WebApi[F],
      reportingRoutes: ReportingHttpRoutes[F]
  )(implicit scheduler: Scheduler): F[fs2.Stream[F, ExitCode]] =
    for {
      event              <- EventsInfo.service[F]
      reportingRoutesOpt = if (reporting) reportingRoutes else HttpRoutes.empty
      baseRoutes = Map(
        "/metrics"   -> corsPolicy(NewPrometheusReporter.service[F](prometheusReporter)),
        "/version"   -> corsPolicy(VersionInfo.service[F]),
        "/status"    -> corsPolicy(StatusInfo.service[F]),
        "/ws/events" -> corsPolicy(event),
        "/api"       -> corsPolicy(WebApiRoutes.service[F](webApi) <+> reportingRoutesOpt),
        // Web API v1 with OpenAPI schema
        "/api/v1" -> corsPolicy(WebApiRoutesV1.create[F](webApi))
      )
      // Legacy reporting routes
      extraRoutes = if (reporting)
        Map("/reporting" -> corsPolicy(reportingRoutes))
      else
        Map.empty
      allRoutes = baseRoutes ++ extraRoutes
    } yield BlazeServerBuilder[F](scheduler)
      .bindHttp(httpPort, host)
      .withHttpApp(RouterFix(allRoutes.toList: _*).orNotFound)
      .withIdleTimeout(connectionIdleTimeout)
      .withResponseHeaderTimeout(connectionIdleTimeout - 1.second)
      .serve

  def aquireAdminHttpServer[F[_]: ConcurrentEffect: Timer: EventConsumer: Log](
      host: String = "0.0.0.0",
      httpPort: Int,
      connectionIdleTimeout: FiniteDuration,
      webApi: WebApi[F],
      adminWebApiRoutes: AdminWebApi[F],
      reportingRoutes: ReportingHttpRoutes[F]
  )(implicit scheduler: Scheduler): F[fs2.Stream[F, ExitCode]] =
    for {
      event <- EventsInfo.service[F]
      baseRoutes = Map(
        "/api" -> corsPolicy(AdminWebApiRoutes.service[F](adminWebApiRoutes) <+> reportingRoutes),
        // Web API v1 (admin) with OpenAPI schema
        "/api/v1" -> corsPolicy(WebApiRoutesV1.createAdmin[F](webApi, adminWebApiRoutes))
      )
    } yield BlazeServerBuilder[F](scheduler)
      .bindHttp(httpPort, host)
      .withHttpApp(RouterFix(baseRoutes.toList: _*).orNotFound)
      .withResponseHeaderTimeout(connectionIdleTimeout - 1.second)
      .withIdleTimeout(connectionIdleTimeout)
      .serve
}
