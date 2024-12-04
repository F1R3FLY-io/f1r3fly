## Info
The point of this directory is to test RSpace++ against a genesis ceremony startup all the way to finalizing a new block.

## Quickstart 

NOTE: See `docs/paul-brain-dump.md` for more information.

### Starting Node

Assumption: you are in a terminal at the project's root with a successfully build fat `.jar` as described above, and as a consequence, in the reproducible development environment. Monospaced examples starting with `$` below are command invocations in your shell.

`$ rm -fr ~/.rnode`

By default, the node uses `~/.rnode` as its `data-dir`. Let's take advantage of this, and start fresh.

`$ amm`

We need to generate an secp256k1 key pair, getting both keys in hexadecimal form. The Ammonite REPL prompt is `@`.

```
@ import $exec.scripts.playground
@ ECPrivateKey.freshPrivateKey
SLF4J: No SLF4J providers were found.
SLF4J: Defaulting to no-operation (NOP) logger implementation
SLF4J: See https://www.slf4j.org/codes.html#noProviders for further details.
res1: ECPrivateKey = ECPrivateKey(Chunk(View(scodec.bits.ByteVector$AtArray@574b7f4a, 1L, 32L)))

@ res1.publicKey.decompressedHex
res3: String = "04458c90e163dcb143b7e358b160a794a6eb85301518a7f46aa5c6f9514188f876809fbd5607d1d338f9056da9e07026c6b5e9ff56d057797fc4c8b3f2f8be76b3"

@ res1.hex
res4: String = "f004ccc5955cef0b7ae52953d861f749bbc8e18b1484217209777af1b4fced4c"

@ <ctl-d>
```

`$ mkdir -p ~/.rnode/genesis`

For convenience, copy the public key hex value above, and create a file in `~/.rnode/genesis` with that as its name, with `.sk` appended:

`$ hx ~/.rnode/genesis/04458c90e163dcb143b7e358b160a794a6eb85301518a7f46aa5c6f9514188f876809fbd5607d1d338f9056da9e07026c6b5e9ff56d057797fc4c8b3f2f8be76b3.sk`

In the file you're now editing, copy and paste the private key hex value above. Save the file.

Now edit `~/.rnode/genesis/bonds.txt`, and copy and paste the public key above. Put a space and a `1` after it on the same line. Save the file:

`04458c90e163dcb143b7e358b160a794a6eb85301518a7f46aa5c6f9514188f876809fbd5607d1d338f9056da9e07026c6b5e9ff56d057797fc4c8b3f2f8be76b3 1`

Now run the node as described above, not specifying `data-dir` explicitly and providing the above private key:

`$ java -Djna.library.path=./rust_libraries --add-opens java.base/sun.security.util=ALL-UNNAMED --add-opens java.base/java.nio=ALL-UNNAMED --add-opens java.base/sun.nio.ch=ALL-UNNAMED -jar node/target/scala-2.12/rnode-assembly-1.0.0-SNAPSHOT.jar run -s --no-upnp --allow-private-addresses --synchrony-constraint-threshold=0.0 --validator-private-key f004ccc5955cef0b7ae52953d861f749bbc8e18b1484217209777af1b4fced4c`

The logs should include:

`Approved block not found, taking part in ceremony as ceremony master`

and one "Bond loaded" line, e.g.

`Bond loaded 04458c90e163dcb143b7e358b160a794a6eb85301518a7f46aa5c6f9514188f876809fbd5607d1d338f9056da9e07026c6b5e9ff56d057797fc4c8b3f2f8be76b3 => 1`

You should also see:

`Starting execution of ApprovedBlockProtocol. Waiting for 0 approvals from genesis validators.`
`Self-approving genesis block.`
`Finished execution of ApprovedBlockProtocol`
`Making a transition to Running state. Approved Block #0 (644386340c...) with empty parents (supposedly genesis)`

These indicate that the node is not waiting for other validators, is unilaterally approving the genesis block, and is now ready to do work to be validated by the one bonded validator in the network—itself.

### Interacting with Node

Run `sbt "nodeCli/run"`. This will perform the remaining steps in `docs/paul_brain_dump.md` using Scala.

NOTE: To start a genesis ceremoney each time, you need to delete relative files. You can run: `rm -rf ~/.rnode/casperbuffer/ ~/.rnode/dagstorage/ ~/.rnode/deploystorage/ ~/.rnode/blockstorage/ ~/.rnode/rnode.log ~/.rnode/rspace++/`