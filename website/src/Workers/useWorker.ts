import { useCallback, useRef, MutableRefObject } from 'react';
import { useMountEffect } from '../definitions';

// @ts-ignore
// eslint-disable-next-line import/no-webpack-loader-syntax
import WebpackWorker from "worker-loader!*"

export interface WorkerInterface {
  /**
   * Identifies the kind of message sent to or received by the Web Worker
   */
  tag: any,
  /**
   * The message payload.
   */
  value?: any,
  /**
   * Identifies the transaction.
   * Automatically generated in the range [0, 1] if undefined.
   * (-1 reserved)
   */
  interactionID?: number,
}

export interface WorkerSubscriptionInterface {
  tag: any,
  callback: (output?: any, interactionID?: number) => void,
}

/**
 * This is a hook which provides a typechecked interface with a web worker.
 * 
 * When called, it should be provided a worker and a setup message for the worker.
 * 
 * following is an example of usage:
 * 
 * ```
 * // @ts-ignore ts(2307)
 * // eslint-disable-next-line import/no-webpack-loader-syntax
 * import ProcessingWorker from "worker-loader!./Workers/your.worker"
 * 
 * function component() {
 * 
 *    const [
 *      subscribeToWorkerFor,
 *      updateWorker
 *    ] = useProcessingWorker<WorkerInput, WorkerOutput, WorkerSubscription>(ProcessingWorker, setupMessage);
 * 
 * }
 * ```
 * 
 * Note the three generics passed, which enforce a user-determined message typechecking.
 * 
 * The generics are defined as follows:
 * 
 * ```WorkerInput``` extends the internal ```WorkerInterface``` which may be exported.
 * It is defined as an union type of the allowed messages 
 * ***to*** the web worker in the ```WorkerInterface``` format.
 * 
 * ```WorkerOutput``` extends the internal ```WorkerInterface``` which may be exported.
 * It is defined as an union type of the allowed messages 
 * ***from*** the web worker in the ```WorkerInterface``` format.
 * 
 * ```WorkerSubscription``` extends the internal ```WorkerSubscriptionInterface``` which may be exported.
 * It closely reflects the ```WorkerOutput``` interface as the ```target``` properties must be shared and
 * the ```output``` property in the subscription's ```callback``` must be the union type of 
 * all the ```value``` property types in WorkerOutput.
 * (This is a limitation in the current model as specifying a specific type for each value in the callbacks
 * leads to complications)
 * 
 * @param WebWorker 
 * @param setupMessage 
 */
export function useWorker
  <
    I extends WorkerInterface,
    O extends WorkerInterface,
    S extends WorkerSubscriptionInterface
  >
  (
    WebWorker: typeof WebpackWorker,
    setupMessage: I,
)
  :
  [
    (subscription: S) => void,
    (update: I) => void
  ] {

  /**
   * Keeps track of the identifier of the last message sent to the worker.
   */
  const lastUpdateID = useRef<number>(-1)
  /**
   * Keeps track of all the subscriptions to the worker output.
   */
  const subscriptions = useRef<S[]>([]);

  /**
   * DO NOT ACCESS DIRECTLY.
   * 
   * This worker ref should be accessed through the helper function ```getWorker()```.
   * 
   * This is because ref initial values are re-evaluated at every render,
   * so to avoid unnecessary re-renders it must be spawned outside the ref.
   * This is achieved by allowing a null value and accessing the worker through
   * a function which checks if the value is null, in which case it returns a 
   * newly spawned worker, or !null, in which case it returns the worker.
   */
  const workerRef: MutableRefObject<Worker | null> = useRef<Worker>(null);
  /**
   * A helper function which provides access to the worker.
   */
  const getWorker = useCallback(() => {
    if (workerRef.current === null) {
      workerRef.current = new WebWorker();
    }
    return workerRef.current;
  }, [WebWorker])

  // Setup listener and send first message
  useMountEffect(() => {

    // Define behavior when receiving messages from the worker
    getWorker().onmessage = (event: MessageEvent) => handleWorkerResponse(event);

    // Initialize the worker
    updateWorker(setupMessage)

    return getWorker().terminate;
  })

  return [subscribeToWorkerFor, updateWorker];

  /**
   * Function returned to set callbacks when the worker generates new output.
   * It accepts a Subscription-type object which is used to define to which messages
   * the hook user will listen to, and the callback to be invoked.
   * @param subscription 
   */
  function subscribeToWorkerFor(subscription: S) {
    subscriptions.current.push(subscription);
  }

  /**
   * Function returned to update worker parameters.
   * This callback allows the hook user to update the web worker using
   * a WorkerInput-type object.
   * @param update 
   */
  function updateWorker(update: I) {
    if (!update.interactionID && update.interactionID !== 0) {
      update.interactionID = Math.random();
    }

    getWorker().postMessage(update);
    lastUpdateID.current = update.interactionID;
  }

  /**
   * This function handles the worker response and redirect it
   * to the hook subscribers.
   * @param event 
   */
  function handleWorkerResponse(event: MessageEvent) {

    if (!event.data) {
      throw new Error("Error: worker sent invalid data")
    }
    let data = event.data as O;

    const s = subscriptions.current
    for (let i = 0; i < s.length; i++) {
      if (s[i].tag === data.tag) {
        s[i].callback(data.value, data.interactionID);
      }
    }
    return;
  }
}

/**
 * An utility function which provides returns values of the same type to the
 * hook.
 * 
 * This is useful when using context because for best typechecking the types should be
 * possible to infer when the context is defined outside the component where the
 * hook would be able to provide it.
 */
export function getEmptyWorkerContext
  <
    I extends WorkerInterface,
    S extends WorkerSubscriptionInterface
  >
  ()
  : [
    (subscription: S) => void,
    (update: I) => void,
  ] {
  return [() => { }, () => { }];
}