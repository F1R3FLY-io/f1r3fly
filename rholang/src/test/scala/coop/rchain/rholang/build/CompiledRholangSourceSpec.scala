package coop.rchain.rholang.build

import coop.rchain.models.rholang_scala_rust_types.SourceToAdtParams
import coop.rchain.models.Par
import org.scalatest.{FlatSpec, Matchers}
import coop.rchain.rholang.build.CompiledRholangSource.sourceToAdt

class CompiledRholangSourceSpec extends FlatSpec with Matchers {

  "CompiledRholangSource" should "correctly call sourceToAdt and return Par" in {
    val testCode = """
                     |new x in {
                     |  x!("Hello, world!")
                     |}
    """.stripMargin

    val params = SourceToAdtParams(source = testCode, normalizerEnv = Map.empty)

    val actualResult = sourceToAdt(params)

    actualResult shouldBe a[Par]
  }
}
