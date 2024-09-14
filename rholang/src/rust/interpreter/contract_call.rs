use models::rhoapi::{ListParWithRandom, Par};

use super::{dispatch::RhoDispatch, rho_runtime::RhoTuplespace};

/**
 * This is a tool for unapplying the messages sent to the system contracts.
 *
 * The unapply returns (Producer, Seq[Par]).
 *
 * The Producer is the function with the signature (Seq[Par], Par) => F[Unit] which can be used to send a message
 * through a channel. The first argument with type Seq[Par] is the content of the message and the second argument is
 * the channel.
 *
 * Note that the random generator and the sequence number extracted from the incoming message are required for sending
 * messages back to the caller so they are given as the first argument list to the produce function.
 *
 * The Seq[Par] returned by unapply contains the message content and can be further unapplied as needed to match the
 * required signature.
 *
 * @param space the rspace instance
 * @param dispatcher the dispatcher
 *
 * See rholang/src/main/scala/coop/rchain/rholang/interpreter/ContractCall.scala
 */
pub struct ContractCall {
    pub space: RhoTuplespace,
    pub dispatcher: RhoDispatch,
}

pub type Producer = Box<dyn FnOnce(Vec<Par>, Par) -> Box<dyn futures::Future<Output = ()>>>;

impl ContractCall {
    pub fn unapply(&self, contract_args: Vec<ListParWithRandom>) -> Option<(Producer, Vec<Par>)> {
        if contract_args.len() == 1 {
            let (args, rand) = (
                contract_args[0].pars.clone(),
                contract_args[0].random_state.clone(),
            );

            let space = self.space.clone();
            let dispatcher = self.dispatcher.clone();
            let produce = Box::new(move |values: Vec<Par>, ch: Par| {
                let space = space.clone();
                let rand = rand.clone();
                Box::new(async move {
                    let produce_result = space.lock().unwrap().produce(
                        ch,
                        ListParWithRandom {
                            pars: values,
                            random_state: rand,
                        },
                        false,
                    );

                    match produce_result {
                        Some((cont, channels)) => {
                            dispatcher
                                .lock()
                                .unwrap()
                                .dispatch(
                                    cont.continuation,
                                    channels.iter().map(|c| c.matched_datum.clone()).collect(),
                                )
                                .await
                                .map_err(|err| panic!("{}", err))
                                .unwrap();
                        }

                        None => {}
                    }
                }) as Box<dyn futures::Future<Output = ()>>
            });

            Some((produce, args.clone()))
        } else {
            None
        }
    }
}
