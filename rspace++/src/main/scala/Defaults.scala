package rspacePlusPlus

import coop.rchain.models._
import coop.rchain.models.Expr.ExprInstance.{GInt, GString}
import scala.collection.immutable.{BitSet}
import coop.rchain.models.TaggedContinuation.TaggedCont.ParBody

class Defaults {
  def arbitraryGInt: GInt = {
    // val v = Gen.posNum[Int]
    val v = 10
    GInt(v.toLong)
  }

  def onePar(i: GInt) = List(
    Par(
      List(),
      List(),
      List(),
      List(Expr(i)),
      List(),
      List(),
      List(),
      List(),
      AlwaysEqual(BitSet()),
      false
    )
  )

  val myPars = onePar(arbitraryGInt)

  val myPar = Par(
    List(),
    List(),
    List(),
    List(Expr(arbitraryGInt)),
    List(),
    List(),
    List(),
    List(),
    AlwaysEqual(BitSet()),
    false
  )

  val myBindPattern = BindPattern(
    onePar(arbitraryGInt),
    None,
    0
  )

  val myListParWithRandom = ListParWithRandom(
    onePar(arbitraryGInt)
  )

  // NOTE: Removed 'r' from code pulled from 'rspace-bench/**/BasicBench.scala' which refers to Blake2b512Random
  val myTaggedContinuation = TaggedContinuation(
    ParBody(
      ParWithRandom(
        Par(
          Vector(
            Send(
              Par(
                Vector(),
                Vector(),
                Vector(),
                List(Expr(GInt(2))),
                Vector(),
                Vector(),
                Vector(),
                List(),
                AlwaysEqual(BitSet()),
                false
              ),
              Vector(
                Par(
                  Vector(),
                  Vector(),
                  Vector(),
                  List(Expr(GString("OK"))),
                  Vector(),
                  Vector(),
                  Vector(),
                  List(),
                  AlwaysEqual(BitSet()),
                  false
                )
              ),
              false,
              AlwaysEqual(BitSet()),
              false
            )
          ),
          Vector(),
          Vector(),
          List(),
          Vector(),
          Vector(),
          Vector(),
          List(),
          AlwaysEqual(BitSet()),
          false
        )
      )
    )
  )
}
