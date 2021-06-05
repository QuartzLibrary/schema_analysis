import {
  WIn, WInTag, WOutTag, DataType, WOut, AllOutputs
} from "../definitions";

type TypecheckedWorker = {
  postMessage: (message: WOut) => void,
  onmessage: ((this: Worker, ev: MessageEvent) => any) | null
}

// @ts-ignore
// eslint-disable-next-line no-restricted-globals
const w: TypecheckedWorker = self as any;

console.log("[WW] Web Worker is ready")

let wasm: typeof import("schema_analysis_wasm") | undefined = undefined;
const get_wasm = async () => {
  if (wasm === undefined) {
    wasm = await import('schema_analysis_wasm');
    console.log("[WW] WASM module is ready")
    return wasm;
  }
  return wasm;
}


/**
 * The function that will handle messages from the parent thread.
 */
w.onmessage = async (event: MessageEvent) => {

  // The data should be of type WorkerInput
  const data = event.data as WIn;

  // Serialized objects lose some of their feature (like methods),
  // if any has been passed to the worker they must be rebuilt.
  switch (data.tag) {

    case WInTag.Start:
      w.postMessage({ tag: WOutTag.Ready })
      break;

    case WInTag.Infer:
      let [urls, dataType] = data.value;
      w.postMessage({
        tag: WOutTag.Inferred,
        value: await handle_inference(urls, dataType),
      })
      break;

    case WInTag.Clear:
      (await get_wasm()).clear_schema()
      w.postMessage({ tag: WOutTag.Cleared })
      break;

    case WInTag.GetAll:
      w.postMessage({
        tag: WOutTag.AllOutputs,
        value: {
          jsonSchema: await handle_json_schema(),
          rust: await handle_rust_types(),
          kotlinJackson: await handle_kotlin_jackson(),
          kotlinKotlinx: await handle_kotlin_kotlinx(),
          typescript: await handle_typescript(),
          typescriptTypeAlias: await handle_typescript_type_alias(),
          rawSchema: await handle_raw(),
        } as AllOutputs,
      })
      break;

    default:
      console.log("[WW] Received an invalid message.");
      console.log(event);
  }

  console.log(`[WW] Responded to: ${WInTag[data.tag]}`)
};

async function handle_inference(urls: string[], dataType: DataType): Promise<[boolean, string | undefined]> {
  let wasm = await get_wasm();

  let success = true;
  let error = undefined;

  for (const url of urls) {

    try {

      console.log(`[WW] Retrieving data at: ${url}`)
      const data: ArrayBuffer = await (await fetch(url)).arrayBuffer()

      try {
        wasm.infer(new Uint8Array(data), dataType);
      } catch (e) { // String
        console.log(e)
        success = false;
        error = `This error occurred while processing the data: \n${e}`;
      }

    } catch (e) {
      console.log(e);
      success = false;
      error = `This error occurred while fetching the provided data: \n${e}`
    }

  }

  return [success, error];
}

async function handle_json_schema(): Promise<string | null> {
  let wasm = await get_wasm();
  try {
    return wasm.get_json_schema();
  } catch (e) {
    console.log("Error while recovering Json Schema: ", e);
    return null;
  }
}

async function handle_rust_types(): Promise<string | null> {
  let wasm = await get_wasm();
  try {
    return wasm.get_rust_types();
  } catch (e) {
    console.log("Error while recovering Rust Types: ", e);
    return null;
  }
}

async function handle_kotlin_jackson(): Promise<string | null> {
  let wasm = await get_wasm();
  try {
    return wasm.get_kotlin_jackson_types();
  } catch (e) {
    console.log("Error while recovering Kotlin Jackson: ", e);
    return null;
  }
}

async function handle_kotlin_kotlinx(): Promise<string | null> {
  let wasm = await get_wasm();
  try {
    return wasm.get_kotlin_kotlinx_types();
  } catch (e) {
    console.log("Error while recovering Kotlin Kotlinx: ", e);
    return null;
  }
}

async function handle_typescript(): Promise<string | null> {
  let wasm = await get_wasm();
  try {
    return wasm.get_typescript_types();
  } catch (e) {
    console.log("Error while recovering Typescript: ", e);
    return null;
  }
}

async function handle_typescript_type_alias(): Promise<string | null> {
  let wasm = await get_wasm();
  try {
    return wasm.get_typescript_type_alias_types();
  } catch (e) {
    console.log("Error while recovering Typescript Type Alias: ", e);
    return null;
  }
}

async function handle_raw(): Promise<string | null> {
  let wasm = await get_wasm();
  try {
    return wasm.get_raw();
  } catch (e) {
    console.log("Error while recovering Raw Representation: ", e);
    return null;
  }
}