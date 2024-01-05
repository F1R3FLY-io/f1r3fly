package coop.rchain.node.configuration.commandline
import org.scalatest.{Matchers, PropSpec}
import org.scalatest.prop.GeneratorDrivenPropertyChecks

class Base16ConverterSpec extends PropSpec with GeneratorDrivenPropertyChecks with Matchers {
  property("parse returns error for bad characters") {

    forAll { s: String =>
      val args = List(("", List(s)))

      val hasInvalidCharacters = s.replaceAll("[^0-9A-Fa-f]", "") != s

      Base16Converter.parse(args).isLeft should be(hasInvalidCharacters)
    }
  }
}
