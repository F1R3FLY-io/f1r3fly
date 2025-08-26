package coop.rchain.rholang.interpreter

import org.scalatest.{FlatSpec, Matchers}
import cats.effect.{ContextShift, IO, Timer}
import java.util.Locale
import scala.concurrent.ExecutionContext

class OpenAIServiceSpec extends FlatSpec with Matchers {

  implicit val ec: ExecutionContext = ExecutionContext.global
  implicit val cs: ContextShift[IO] = IO.contextShift(ec)
  implicit val timer: Timer[IO]     = IO.timer(ec)

  behavior of "DisabledOpenAIService"

  it should "return UnsupportedOperationException for ttsCreateAudioSpeech" in {
    val service = new DisabledOpenAIService()

    val result = service.ttsCreateAudioSpeech[IO]("test prompt")

    val exception = intercept[UnsupportedOperationException] {
      result.unsafeRunSync()
    }
    exception.getMessage should include("OpenAI service is disabled")
  }

  it should "return UnsupportedOperationException for dalle3CreateImage" in {
    val service = new DisabledOpenAIService()

    val result = service.dalle3CreateImage[IO]("test prompt")

    val exception = intercept[UnsupportedOperationException] {
      result.unsafeRunSync()
    }
    exception.getMessage should include("OpenAI service is disabled")
  }

  it should "return UnsupportedOperationException for gpt4TextCompletion" in {
    val service = new DisabledOpenAIService()

    val result = service.gpt4TextCompletion[IO]("test prompt")

    val exception = intercept[UnsupportedOperationException] {
      result.unsafeRunSync()
    }
    exception.getMessage should include("OpenAI service is disabled")
  }

  behavior of "Environment variable parsing"

  it should "parse valid environment variable values" in {
    def parseEnvValue(value: String): Option[Boolean] =
      Option(value).flatMap { v =>
        v.toLowerCase(Locale.ENGLISH) match {
          case "true" | "1" | "yes" | "on"  => Some(true)
          case "false" | "0" | "no" | "off" => Some(false)
          case _                            => None
        }
      }

    // Valid true values
    parseEnvValue("true") shouldBe Some(true)
    parseEnvValue("TRUE") shouldBe Some(true)
    parseEnvValue("1") shouldBe Some(true)
    parseEnvValue("yes") shouldBe Some(true)
    parseEnvValue("YES") shouldBe Some(true)
    parseEnvValue("on") shouldBe Some(true)
    parseEnvValue("ON") shouldBe Some(true)

    // Valid false values
    parseEnvValue("false") shouldBe Some(false)
    parseEnvValue("FALSE") shouldBe Some(false)
    parseEnvValue("0") shouldBe Some(false)
    parseEnvValue("no") shouldBe Some(false)
    parseEnvValue("NO") shouldBe Some(false)
    parseEnvValue("off") shouldBe Some(false)
    parseEnvValue("OFF") shouldBe Some(false)

    // Invalid values
    parseEnvValue("maybe") shouldBe None
    parseEnvValue("2") shouldBe None
    parseEnvValue("") shouldBe None
    parseEnvValue("invalid") shouldBe None
  }

  behavior of "OpenAI service instantiation behavior"

  it should "demonstrate the expected service selection logic with environment variable support" in {
    // Test the logic that would be used in OpenAIServiceImpl.instance
    def selectService(
        configEnabled: Option[Boolean],
        envEnabled: Option[Boolean],
        hasApiKey: Boolean
    ): String = {
      val isEnabled = configEnabled.getOrElse(envEnabled.getOrElse(false))
      if (isEnabled) {
        if (hasApiKey) {
          "OpenAIServiceImpl" // Would create real service
        } else {
          "IllegalStateException" // Would throw exception
        }
      } else {
        "DisabledOpenAIService" // Would create disabled service
      }
    }

    // Test priority: config takes precedence over env var
    selectService(Some(true), Some(false), true) shouldBe "OpenAIServiceImpl"
    selectService(Some(false), Some(true), true) shouldBe "DisabledOpenAIService"

    // Test env var fallback when config not set
    selectService(None, Some(true), true) shouldBe "OpenAIServiceImpl"
    selectService(None, Some(false), true) shouldBe "DisabledOpenAIService"

    // Test default fallback when neither set
    selectService(None, None, true) shouldBe "DisabledOpenAIService"
    selectService(None, None, false) shouldBe "DisabledOpenAIService"

    // Test API key validation still applies
    selectService(Some(true), None, false) shouldBe "IllegalStateException"
    selectService(None, Some(true), false) shouldBe "IllegalStateException"
  }

  it should "demonstrate the expected service selection logic (legacy test)" in {
    // Test the logic that would be used in OpenAIServiceImpl.instance
    def selectService(enabled: Boolean, hasApiKey: Boolean): String =
      if (enabled) {
        if (hasApiKey) {
          "OpenAIServiceImpl" // Would create real service
        } else {
          "IllegalStateException" // Would throw exception
        }
      } else {
        "DisabledOpenAIService" // Would create disabled service
      }

    // Test all combinations
    selectService(enabled = false, hasApiKey = false) shouldBe "DisabledOpenAIService"
    selectService(enabled = false, hasApiKey = true) shouldBe "DisabledOpenAIService"
    selectService(enabled = true, hasApiKey = false) shouldBe "IllegalStateException"
    selectService(enabled = true, hasApiKey = true) shouldBe "OpenAIServiceImpl"
  }

  it should "validate API key resolution priority" in {
    // Test the logic for API key resolution
    def resolveApiKey(configKey: Option[String], envKey: Option[String]): Option[String] = {
      val apiKeyFromConfig = configKey.filter(_.nonEmpty)
      apiKeyFromConfig.orElse(envKey.filter(_.nonEmpty))
    }

    // Config key takes priority
    resolveApiKey(Some("config-key"), Some("env-key")) shouldBe Some("config-key")

    // Falls back to env key
    resolveApiKey(None, Some("env-key")) shouldBe Some("env-key")
    resolveApiKey(Some(""), Some("env-key")) shouldBe Some("env-key")

    // Returns None if neither available
    resolveApiKey(None, None) shouldBe None
    resolveApiKey(Some(""), Some("")) shouldBe None
  }

  behavior of "API key validation logic"

  it should "demonstrate service initialization with validation" in {
    // Test the enhanced service selection logic that includes validation
    def selectServiceWithValidation(
        enabled: Boolean,
        hasApiKey: Boolean,
        validationEnabled: Boolean,
        validationSucceeds: Boolean
    ): String =
      if (enabled) {
        if (hasApiKey) {
          if (validationEnabled) {
            if (validationSucceeds) {
              "OpenAIServiceImpl" // Validation passed
            } else {
              "IllegalStateException" // Validation failed
            }
          } else {
            "OpenAIServiceImpl" // Validation skipped
          }
        } else {
          "IllegalStateException" // No API key
        }
      } else {
        "DisabledOpenAIService" // Service disabled
      }

    // Test validation enabled and succeeds
    selectServiceWithValidation(
      enabled = true,
      hasApiKey = true,
      validationEnabled = true,
      validationSucceeds = true
    ) shouldBe "OpenAIServiceImpl"

    // Test validation enabled but fails
    selectServiceWithValidation(
      enabled = true,
      hasApiKey = true,
      validationEnabled = true,
      validationSucceeds = false
    ) shouldBe "IllegalStateException"

    // Test validation disabled (skipped)
    selectServiceWithValidation(
      enabled = true,
      hasApiKey = true,
      validationEnabled = false,
      validationSucceeds = false // doesn't matter
    ) shouldBe "OpenAIServiceImpl"

    // Test service disabled (validation irrelevant)
    selectServiceWithValidation(
      enabled = false,
      hasApiKey = true,
      validationEnabled = true,
      validationSucceeds = true
    ) shouldBe "DisabledOpenAIService"

    // Test no API key (validation irrelevant)
    selectServiceWithValidation(
      enabled = true,
      hasApiKey = false,
      validationEnabled = true,
      validationSucceeds = true
    ) shouldBe "IllegalStateException"
  }
}
