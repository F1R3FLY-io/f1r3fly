package coop.rchain.rholang.interpreter

import akka.actor.ActorSystem
import akka.stream.Materializer
import akka.stream.scaladsl.Sink
import io.cequence.openaiscala.domain.{ModelId, UserMessage}
import io.cequence.openaiscala.domain.settings.{
  CreateChatCompletionSettings,
  CreateCompletionSettings,
  CreateImageSettings,
  CreateSpeechSettings,
  VoiceType
}
import io.cequence.openaiscala.service.{OpenAIService => CeqOpenAIService, OpenAIServiceFactory}
import com.typesafe.config.ConfigFactory
import com.typesafe.scalalogging.Logger
import cats.effect.{Concurrent, Sync}
import cats.syntax.all._

import java.util.Locale
import java.util.concurrent.Executors
import scala.concurrent.{ExecutionContext, Future}

trait OpenAIService {

  /** @return raw mp3 bytes */
  def ttsCreateAudioSpeech[F[_]](prompt: String)(
      implicit F: Concurrent[F]
  ): F[Array[Byte]]

  /** @return ULR to a generated image */
  def dalle3CreateImage[F[_]](prompt: String)(
      implicit F: Concurrent[F]
  ): F[String]

  def gpt4TextCompletion[F[_]](prompt: String)(
      implicit F: Concurrent[F]
  ): F[String]
}

class DisabledOpenAIService extends OpenAIService {

  private[this] val logger: Logger = Logger[this.type]

  def ttsCreateAudioSpeech[F[_]](prompt: String)(implicit F: Concurrent[F]): F[Array[Byte]] = {
    logger.debug("OpenAI service is disabled - ttsCreateAudioSpeech request ignored")
    F.raiseError(new UnsupportedOperationException("OpenAI service is disabled via configuration"))
  }

  def dalle3CreateImage[F[_]](prompt: String)(implicit F: Concurrent[F]): F[String] = {
    logger.debug("OpenAI service is disabled - dalle3CreateImage request ignored")
    F.raiseError(new UnsupportedOperationException("OpenAI service is disabled via configuration"))
  }

  def gpt4TextCompletion[F[_]](prompt: String)(implicit F: Concurrent[F]): F[String] = {
    logger.debug("OpenAI service is disabled - gpt4TextCompletion request ignored")
    F.raiseError(new UnsupportedOperationException("OpenAI service is disabled via configuration"))
  }
}

class OpenAIServiceImpl extends OpenAIService {

  private[this] val logger: Logger = Logger[this.type]

  implicit private val ec: ExecutionContext =
    ExecutionContext.fromExecutor(Executors.newCachedThreadPool())
  private val system                              = ActorSystem()
  implicit private val materializer: Materializer = Materializer(system)

  // Build OpenAI client at startup.
  // The API key is resolved in the following order:
  //   1. From Typesafe configuration path `openai.api-key` if defined (e.g. in `rnode.conf` or `defaults.conf`).
  //   2. Fallback to environment variable `OPENAI_SCALA_CLIENT_API_KEY` (to preserve backward compatibility).
  //   3. Throw an exception if the key is missing (crash the node at startup).
  private val openAIService: CeqOpenAIService = {
    val config = ConfigFactory.load()

    // Read from config if present.
    val apiKeyFromConfig: Option[String] =
      if (config.hasPath("openai.api-key")) Some(config.getString("openai.api-key")) else None

    // Fallback to env variable (legacy behaviour)
    val apiKey: Option[String] =
      apiKeyFromConfig.orElse(Option(System.getenv("OPENAI_SCALA_CLIENT_API_KEY")))

    apiKey match {
      case Some(key) if key.nonEmpty =>
        logger.info("OpenAI service initialized successfully")
        OpenAIServiceFactory(key)
      case _ =>
        throw new IllegalStateException(
          "OpenAI API key is not configured. Provide it via config path 'openai.api-key' " +
            "or env var OPENAI_SCALA_CLIENT_API_KEY."
        )
    }
  }

  // shutdown system before jvm shutdown
  sys.addShutdownHook {
    system.terminate()
  }

  def ttsCreateAudioSpeech[F[_]](prompt: String)(
      implicit F: Concurrent[F]
  ): F[Array[Byte]] = {
    val future: Future[Array[Byte]] = openAIService
      .createAudioSpeech(
        prompt,
        CreateSpeechSettings(
          model = ModelId.tts_1_1106,
          voice = VoiceType.shimmer
        )
      )
      .flatMap(
        response =>
          response.map(_.toByteBuffer.array()).runWith(Sink.fold(Array.emptyByteArray)(_ ++ _))
      )

    F.async[Array[Byte]] { cb =>
      future.onComplete {
        case scala.util.Success(response) =>
          logger.info("OpenAI createAudioSpeech request succeeded")
          cb(Right(response))
        case scala.util.Failure(e) =>
          logger.warn("OpenAI createAudioSpeech request failed", e)
          cb(Left(e))
      }
    }
  }

  def dalle3CreateImage[F[_]](prompt: String)(
      implicit F: Concurrent[F]
  ): F[String] = {
    val future: Future[String] = openAIService
      .createImage(
        prompt,
        CreateImageSettings(
          model = Some(ModelId.dall_e_3),
          n = Some(1)
        )
      )
      .map { response =>
        // TODO: handle error properly
        response.data.headOption.flatMap(_.get("url")).get
      }

    F.async[String] { cb =>
      future.onComplete {
        case scala.util.Success(response) =>
          logger.info("OpenAI createImage request succeeded")
          cb(Right(response))
        case scala.util.Failure(e) =>
          logger.warn("OpenAI createImage request failed", e)
          cb(Left(e))
      }
    }
  }

  def gpt4TextCompletion[F[_]](prompt: String)(
      implicit F: Concurrent[F]
  ): F[String] = {
    val future: Future[String] = openAIService
      .createChatCompletion(
        Seq(UserMessage(prompt)),
        CreateChatCompletionSettings(
          model = ModelId.gpt_4_1_mini,
          top_p = Some(0.5),
          temperature = Some(0.5)
        )
      )
      .map(response => response.choices.head.message.content)

    F.async[String] { cb =>
      future.onComplete {
        case scala.util.Success(response) =>
          logger.info("OpenAI gpt4 request succeeded")
          cb(Right(response))
        case scala.util.Failure(e) =>
          logger.warn("OpenAI gpt4 request failed", e)
          cb(Left(e))
      }
    }
  }
}

object OpenAIServiceImpl {

  private[this] val logger: Logger = Logger[this.type]

  /**
    * Provides the appropriate OpenAI service based on configuration and environment.
    * Priority order for enabling OpenAI:
    *   1. Configuration: openai.enabled = true/false
    *   2. Environment variable: OPENAI_ENABLED = true/false
    *   3. Default: false (disabled for safety)
    * - If enabled, returns OpenAIServiceImpl (will crash at startup if no API key)
    * - If disabled, returns DisabledOpenAIService
    */
  lazy val instance: OpenAIService = {
    val config = ConfigFactory.load()

    // Check configuration first (highest priority)
    val configEnabled = if (config.hasPath("openai.enabled")) {
      Some(config.getBoolean("openai.enabled"))
    } else {
      None
    }

    // Check environment variable as fallback
    val envEnabled = Option(System.getenv("OPENAI_ENABLED")).flatMap { value =>
      value.toLowerCase(Locale.ENGLISH) match {
        case "true" | "1" | "yes" | "on"  => Some(true)
        case "false" | "0" | "no" | "off" => Some(false)
        case _                            => None // Invalid env var value, ignore it
      }
    }

    // Resolve final enabled state: config takes priority, then env, then default false
    val isEnabled = configEnabled.getOrElse(envEnabled.getOrElse(false))

    if (isEnabled) {
      val source = if (configEnabled.isDefined) "configuration" else "environment variable"
      logger.info(s"OpenAI service is enabled via $source - initializing with API key validation")
      new OpenAIServiceImpl // This will crash at startup if no API key is provided
    } else {
      val source =
        if (configEnabled.isDefined) "configuration"
        else if (envEnabled.isDefined) "environment variable"
        else "default (not configured)"
      logger.info(s"OpenAI service is disabled via $source")
      new DisabledOpenAIService
    }
  }

  /** @deprecated Use `instance` instead. Kept for backward compatibility. */
  lazy val realOpenAIService: OpenAIService = instance
}
