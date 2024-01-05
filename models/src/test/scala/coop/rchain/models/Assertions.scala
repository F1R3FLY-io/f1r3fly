package coop.rchain.models
import org.scalatest.{Assertion, Assertions => ScalaTestAssertions}

object Assertions extends ScalaTestAssertions {

  def assertEqual[A: Pretty](result: A, expected: A): Assertion =
    assert(
      result == expected,
      LazyClue(
        s"""
         |
         |Actual value:
         |
         |${Pretty.pretty(result)}
         |
         |was not equal to expected:
         |
         |${Pretty.pretty(expected)}
         |
         |""".stripMargin
      )
    )

}
