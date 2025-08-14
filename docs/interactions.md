# Axes

persistent / transient
concurrent / sequential

# Acknowledged read

    ⟦for(ptrn <- x?!){P}⟧ 
    =
    for((ptrn, r) <- x){ r!() | ⟦P⟧ }

    ⟦for(ptrn <= x?!){P}⟧ 
    =
    for((ptrn, r) <= x){ r!() | ⟦P⟧ }

    ⟦for(ptrn <<- x?!){P}⟧ 
    =
    for((ptrn, r) <<- x){ r!() | ⟦P⟧ }

# Method call

## Simple

    One send, one receive.  If the agent has many listeners on x, they race to receive the message.
      If the listener that wins sends multiple times on r, they race to send the message.
    ⟦for(ptrn <- x!?(a1, ..., ak)){P}⟧
    =
    new r in { x!(a1, ..., ak, *r) | for(ptrn <- r){ ⟦P⟧ } }

    One send, many receive.  The agent may send multiple times on the return channel;
      this uses the same handler for all of the sends.
    ⟦for(ptrn <= x!?(a1, ..., ak)){P}⟧
    =
    new r in { x!(a1, ..., ak, *r) | for(ptrn <= r){ ⟦P⟧ } }

    Many send, one receive.  If the agent has many listeners on x or if the listener on x
      is nondeterministic in its response, they race to be the first responder.
      Note that it creates a single return channel!
    ⟦for(ptrn <- x!!?(a1, ..., ak)){P}⟧
    =
    new r in { x!!(a1, ..., ak, *r) | for(ptrn <- r){ ⟦P⟧ } }

    Can't do both
    ⟦for(ptrn <= x!!?(a1, ..., ak)){P}⟧ = Invalid

    One/peek is useless; get same behavior as <- x!?
    Many/peek is similarly useless; get same behavior as <- x!!?

## Sugar for ignored response

    ⟦x!?(a1, ..., ak);P⟧
    =
    ⟦for(..._ <- x!?(a1, ..., ak)){P}⟧

## Sugar for no continuation

    ⟦x!?(a1, ..., ak).⟧
    =
    ⟦for(..._ <- x!?(a1, ..., ak)){ Nil }⟧

## Obvious extensions with join (&) and sequence (;)

Sequence is nesting.  Join is commutative, so we only need to define the result on the first receipt, as above.

    for(
        ptrn11 <- x11!?(...) & ... & ptrnm1 <- xm1!?(...) ;
        ...
    ) { P }

# Agents with method definitions

Note method names can be repeated, running in parallel

    ⟦agent foo(...fooPtrns) {
      method fooMethod1(...ptrns1) { P1 } |
      ... |
      method fooMethodn(...ptrnsn) { Pn } |
      default() { Q }
    }⟧
    =
    contract foo(...fooArgs, rFoo) = {
      new this in {
        // Proper call to this method
        for(@"fooMethod1", ...ptrns1, r <= this) { ⟦P1⟧{r/return} } |
        // Improper call to this method but has return
        for(@"fooMethod1", ..._, r <= this) { ⟦Q⟧{r/return} } |
        // Improper call to this method with no return
        for(@"fooMethod1" <= this) { Nil } |
        ...
        for(@"fooMethodn", ...ptrnsn, r <= this) { ⟦Pn⟧{r/return} } |
        for(@"fooMethodn", ..._, r <= this) { ⟦Q⟧{r/return} } |
        for(@"fooMethodn") <= this { Nil } |
        
        // Handle calls to nonexistent methods with return channel
        for(~@"fooMethod1" /\ ... /\ ~@"fooMethodn", ..._, r <= this) { ⟦Q⟧{r/return} } | 
        // Handle calls to nonexistent methods without return channel
        for(~@"fooMethod1" /\ ... /\ ~@"fooMethodn" <= this) { Nil } | 
        // Handle empty method
        for(<= this) { Nil } | 
      } |
      rFoo!(bundle+{this})
    }

Then x!? syntax works with both constructor and methods.

    for(barInstance <- bar!?(...barArgs)) {
      barInstance!?(@"update", ...updateArgs);
      for(response <- barInstance!?(@"compute", ...computeArgs)){
        // etc.
      }
    }

Possible "call" sugar:

    ⟦x!y(...args)⟧ = x!?(@"y", ...args)

# Let

## Simple

The processes can be replaced by Nil if any ptrn doesn't match.  v2 and ptrn2 can mention vars in ptrn1

    ⟦let ptrn1 <- v1 ; ptrn2 <- v2 ; ... ; ptrnn <- vn = in P⟧
    =
    new x1 in { x1!(v1) | for(ptrn1 <- x1) { ⟦let ptrn2 <- v2 ; ... ; ptrnn <- vn in P ⟧ } }
    =
    ⟦P⟧{v1/ptrn1 ; ... ; vn/ptrnn}

This is the same as

    v1 match {
       case ptrn => ⟦let ptrn2 <- v2 ; ... ; ptrnn <- vn in P ⟧
    }

Parallel let. v2 and ptrn2 can't mention vars in ptrn1

    ⟦let ptrn1 <- v1 & ptrn2 <- v2 & ... & ptrnn <- vn = in P⟧
    =
    new x1, ..., xn in { 
       x1!(v1) | ... | xn!(vn) | 
       for(ptrn1 <- x1 & ... & ptrnn <- xn) { ⟦ P ⟧ }
    }
    =
    ⟦P⟧{v1/ptrn1 & ... & vn/ptrnn}

These are preparatory for letrec.

# Acknowledged read / method call interaction for nullary call

Acknowledged read and method call aren't intended to work together, but they do if the call is nullary.

⟦for(<- x?!){P} | x!?();Q | x!?();R⟧
=
for(r <- x){ r!() | ⟦P⟧ } | ⟦for(..._ <- x!?()){ ⟦Q⟧ } | ⟦for(..._ <- x!?()){ ⟦R⟧ }⟧
=
for(r1 <- x){ r1!() | ⟦P⟧ } | new r2 in { x!(*r2) | for(..._ <- r2){ ⟦Q⟧ } } | new r3 in { x!(*r3) | for(..._ <- r3){ ⟦R⟧ } }
=
new r2, r3 in { for(r1 <- x){ r1!() | ⟦P⟧ } | x!(*r2) | for(..._ <- r2){ ⟦Q⟧ } | x!(*r3) | for(..._ <- r3){ ⟦R⟧ } }
=> x!(*r2) wins
new r2, r3 in { r2!() | ⟦P⟧ | for(..._ <- r2){ ⟦Q⟧ } | x!(*r3) | for(..._ <- r3){ ⟦R⟧ } }
=>
new r2, r3 in { ⟦P⟧ | ⟦Q⟧ | x!(*r3) | for(..._ <- r3){ ⟦R⟧ }  }
= since r2, r3 # P
⟦P⟧ | new r2, r3 in { ⟦Q⟧ | x!(*r3) | for(..._ <- r3){ ⟦R⟧ } }
= since r2, r3 # Q
⟦P⟧ | ⟦Q⟧ | new r2, r3 in { x!(*r3) | for(..._ <- r3){ ⟦R⟧ } }
= garbage collection
⟦P⟧ | ⟦Q⟧

and symmetrically for R when x!(*r3) wins.

# Static analysis

Let avoids hitting rspace at all by allowing immediate substitution as above.

The other situation where a name can be transient, allowing immediate substitution, is when the method being called can be proven to use the return channel exactly once.  There'll be some methods where it's easy, some where one would need a proof assistant, and some where it's undecidable.

It's still not clear to me why we need four rspaces.

# Garbage collection

- for(ptrn <- x!?(a1, ..., ak)){P}: listens for exactly one message from r, so after the first synchronization on that channel, garbage collect anything else sending or listening on that channel forever.

- for(ptrn <= x!?(a1, ..., ak)){P}: may or may not be able to garbage collect: the descendants of the listener on x may hold r indefinitely.  If it doesn't, use standard garbage collection to collect new r in { for(ptrn <= r){P} }.

- for(ptrn <- x!!?(a1, ..., ak)){P}: listens for exactly one message from r, so after the first synchronization on that channel, garbage collect anything else sending or listening on that channel forever.

