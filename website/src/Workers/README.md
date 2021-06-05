# On this Web Worker Interface:

**Why:**

This Web Worker interface has two main goals:
- A simpler way to comunicate with the Worker which retaines a high level of flexibility.
- Structured and typechecked communication with the Web Worker.

**How:**

Two interfaces restrict the structure of messages which may be shared:

```Typescript
export interface WorkerInterface {
  /** Identifies the kind of message sent to or received by the Web Worker */
  tag: any,
  /** The message payload. */
  value?: any,
  /** Identifies the message (and response). */
  interactionID?: number,
}

export interface WorkerSubscriptionInterface {
  tag: any,
  callback: (output?: any, interactionID?: number) => void,
}
```

Here the `tag` field is used to identify the kind of message that the Worker or user will receive and may restrict the kind of `value` the message is allowed to have by way of user-defined types.

The interface is then personalized by the user with custom types and the generics of the hook.

Here is an example taken from the project:

```Typescript
// Some tags have been removed for clarity
export enum WInTag {
  Start, Infer,
}
export type WIn = (
  { tag: WInTag.Start } |
  { tag: WInTag.Infer, 
    value: [string[], DataType] }
);

export enum WOutTag {
  Ready, Inferred,
}
export type AllOutputs = { ... }
export type WOut = (
  { tag: WOutTag.Ready, } |
  { tag: WOutTag.Inferred,
    value: [boolean, string | undefined] }
);

export type WSub = (
  { tag: WOutTag.Ready, callback: () => void } | 
  { tag: WOutTag.Inferred,
    callback: (output: [boolean, string | undefined]) => void }
);
```

As it is evident from the examples, messages to and from the worker are restricted to an union type in which available `tag` and `value` pairs are defined, allowing for strict typechecking. `interactionID` is not very useful in this context, but it helps keep track of things when there are multiple messages of the same type going around (it was very helpful for the project this interface was originaly designed for).

**Usage:**
```Typescript
// Module root:
// .ts extension included to pass the tests
// eslint-disable-next-line import/no-webpack-loader-syntax
import ProcessingWorker from "worker-loader!../Workers/processing.worker.ts"

import { WOut, WIn, WSub, WInTag } from '../definitions';

// In the component:
const [
  // (subscription: WSub) => void
  subscribeToWorkerFor,
  // (update: WIn) => void
  updateWorker,
] = useWorker<WIn, WOut, WSub>(
  ProcessingWorker, { tag: WInTag.Start }
);
```

- `subscribeToWorkerFor` will register the subscription and use the callback inside to report any new messages with the correct tag.
- `updateWorker` will send the message/update to the worker.

**Result:**

The upside of this small interface is the ability to use a Worker by first defining the message types and then using the hook to get two typechecked callbacks for subscription or sending messages to the Worker.

**Limitations:**

The main limitations are:
- A bit verbose and kind of clunky.
- While typechecking works, it requires a duplicate effort in the definition of `WOut` and `WSub` which can become tedious to keep up to date if there are a lot of different messages.

**Requirements:**
- [worker-loader](https://github.com/webpack-contrib/worker-loader)