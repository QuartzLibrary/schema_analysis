import React, { useState } from 'react';

import { useWorker } from '../Workers/useWorker';
import {
  WOut, WIn, WSub, DataType, WOutTag, WInTag, useMountEffect, AllOutputs,
} from '../definitions';

// .ts extension included to pass the tests
// eslint-disable-next-line import/no-webpack-loader-syntax
import ProcessingWorker from "worker-loader!../Workers/processing.worker.ts"
import { Display } from './Display';

const DEFAULT_ERROR_MESSAGE = "There was an unknown error, please check the console and open a Github issue.";

export function Result(props: {
  dataUrls: string[],
  dataType: DataType,
}) {
  enum DisplayStatus {
    Loading,
    Error,
    Ready,
  }

  const [status, setStatus] = useState(DisplayStatus.Loading);
  const [errorMessage, setErrorMessage] = useState<string | undefined>();
  const [outputs, setOutputs] = useState<AllOutputs | undefined>();

  // Here we setup the worker, which will be passed on as context later.
  const [
    subscribeToWorkerFor,
    updateWorker,
  ] = useWorker<WIn, WOut, WSub>(ProcessingWorker, { tag: WInTag.Start });

  // Here we subscribe to the worker events and kick off the inference process
  useMountEffect(() => {
    subscribeToWorkerFor({
      tag: WOutTag.Inferred,
      callback: handleWorkerInferred,
    });
    subscribeToWorkerFor({
      tag: WOutTag.AllOutputs,
      callback: handleWorkerOutput,
    });

    updateWorker({
      tag: WInTag.Infer,
      value: [props.dataUrls, props.dataType],
    });
  })

  return (
    <div className="display">

      {(DisplayStatus.Loading === status) ? <div className="loading"><p>Loading...</p></div> : <></>}

      {(DisplayStatus.Error === status) ? <div className="error"><p>
        {errorMessage ? <pre>{errorMessage}</pre> : DEFAULT_ERROR_MESSAGE}
      </p></div> : <></>}

      {(DisplayStatus.Ready === status && outputs) ? <Display outputs={outputs} /> : <></>}

    </div>
  );

  function handleWorkerInferred(output: [boolean, string | undefined]) {
    let [success, errorMessage] = output;
    if (success) {
      console.log("[Main] Inference successful");
      updateWorker({ tag: WInTag.GetAll });
    } else {
      console.log("[Main] Inference unsuccessful");
      setErrorMessage(errorMessage);
      setStatus(DisplayStatus.Error);
      return;
    }
  }

  function handleWorkerOutput(output: AllOutputs) {
    setOutputs(output);
    setStatus(DisplayStatus.Ready);
    console.log(`[Main] Got the outputs back`);
  }
}
