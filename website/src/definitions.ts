import { useEffect, useLayoutEffect } from "react";

/**
 * The currently supported data types.
 * Note: the numeric values should be the same as in the wasm module
 */
export enum DataType {
  // Avoid 0 valus that can lead to problems given Typescript's handling of enums and
  // true/false casting in Javascript.
  Json = 1,
  Yaml = 2,
  Cbor = 3,
  Toml = 4,
  // Bson = 5,
  Xml = 6,
}

export const DATA_TYPES = [
  DataType.Json, DataType.Yaml, DataType.Cbor, DataType.Toml, DataType.Xml
];

export enum WInTag {
  Start,
  Clear,
  Infer,
  GetAll,
}
export type WIn = (
  { tag: WInTag.Start }
  | { tag: WInTag.Clear, }
  | {
    tag: WInTag.Infer,
    value: [string[], DataType],
  }
  | { tag: WInTag.GetAll, }
);

export enum WOutTag {
  Ready,
  Cleared,
  Inferred,
  AllOutputs,
}
export type AllOutputs = {
  jsonSchema?: string,
  rust?: string,
  kotlinJackson?: string,
  kotlinKotlinx?: string,
  typescript?: string,
  typescriptTypeAlias?: string,
  rawSchema?: string,
}
export type WOut = (
  { tag: WOutTag.Ready, }
  | { tag: WOutTag.Cleared, }
  | {
    tag: WOutTag.Inferred,
    /* Whether the analysis was successful or the error message. */
    value: [boolean, string | undefined],
  } | {
    tag: WOutTag.AllOutputs,
    value: AllOutputs,
  }
);

export type WSub = ({
  tag: WOutTag.Ready, callback: () => void,
} | {
  tag: WOutTag.Cleared, callback: () => void,
} | {
  tag: WOutTag.Inferred,
  callback: (output: [boolean, string | undefined]) => void,
} | {
  tag: WOutTag.AllOutputs,
  callback: (output: AllOutputs) => void,
});

/**
 * Exactly the same as useEffect, but with 0 dependencies.
 * Makes the fact that the effect will execute only once upon mounting 
 * (as there are no changing dependency values) clear and avoids linting warnings.
 * @param callback 
 */
// eslint-disable-next-line react-hooks/exhaustive-deps
export const useMountEffect = (callback: () => (void | (() => void | undefined))) => useEffect(callback, [])
/**
 * Exactly the same as useLayoutEffect, but with 0 dependencies.
 * Makes the fact that the effect will execute only once upon mounting 
 * (as there are no changing dependency values) clear and avoids linting warnings.
 * @param callback 
 */
// eslint-disable-next-line react-hooks/exhaustive-deps
export const useMountLayoutEffect = (callback: () => (void | (() => void | undefined))) => useLayoutEffect(callback, [])
