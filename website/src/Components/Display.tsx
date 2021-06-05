import React, { useRef, useState } from 'react';

import SyntaxHighlighter from 'react-syntax-highlighter';
import { a11yDark } from 'react-syntax-highlighter/dist/esm/styles/hljs';

import { AllOutputs, useMountEffect } from '../definitions';
import { downloadBlob } from '../utilities';

enum OutputType {
    JsonSchema,
    RustTypes,
    KotlinJacksonTypes,
    KotlinKotlinxTypes,
    TypescriptTypes,
    TypescriptTypeAliasTypes,
    Raw,
}
const OUTPUT_TYPES = [
    OutputType.Raw,
    OutputType.JsonSchema, OutputType.RustTypes,
    OutputType.KotlinJacksonTypes, OutputType.KotlinKotlinxTypes,
    OutputType.TypescriptTypes, OutputType.TypescriptTypeAliasTypes,
]

export function Display(props: {
    outputs: AllOutputs,
}) {

    const [outputType, setOutputType] = useState(OutputType.Raw);

    // To access the value in the callback.
    let outputTypeRef = useRef<OutputType>(outputType);
    outputTypeRef.current = outputType;

    useMountEffect(() => {
        document.addEventListener("keydown", handleKeyPresses)
    })

    return (
        <>
            <div className="output-options">{OUTPUT_TYPES.map(enumToButton)}</div>
            <div className="output-wrapper">{OUTPUT_TYPES.map(displayOutput)}</div>
            <button className="save-button" onClick={download}><span role="img" aria-label="Save">save</span></button>
        </>
    )

    function enumToButton(t: OutputType) {
        const typeName = outputTypeToName(t);
        return (
            <button
                key={typeName}
                onClick={() => setOutputType(t)}
                className={t === outputType ? "active" : ""}
            >
                {typeName}
            </button>
        )
    }

    function displayOutput(o: OutputType) {
        if (o !== outputType) {
            return <></>
        } else {
            const text = extractOutputTypeText(o, props.outputs);
            return <Output key={OutputType[o]} text={text} type={o} />;
        }
    }

    function handleKeyPresses(event: KeyboardEvent) {
        if (event.ctrlKey && event.key === "s") {
            download();
            event.preventDefault();
        }
    }

    function download() {
        let outputTypeInner = outputTypeRef.current;
        const fileString = extractOutputTypeText(outputTypeInner, props.outputs);
        if (fileString) {
            const blob = new Blob([fileString], { type: "text/plain" });
            const name = OutputType[outputTypeInner].toLowerCase();
            const extension = outputTypeFileExtension(outputTypeInner);
            downloadBlob(blob, `${name}.${extension}`);
        }
    }
}

function Output(props: {
    text?: string,
    type: OutputType,
}) {
    if (props.text === undefined) { return <></> }

    return (<>
        <Credits type={props.type} />
        <SyntaxHighlighter
            key={OutputType[props.type]}
            className="output"
            language={outputTypeToLanguage(props.type)}
            style={a11yDark}
        >
            {props.text}
        </SyntaxHighlighter>
    </>
    )
}

function Credits(props: {
    type: OutputType,
}) {
    const CREDITS_SERDE = <div>
        This analysis was made possible by <a href="https://github.com/serde-rs/serde">Serde</a> and{" "}
        <a href="https://github.com/serde-rs/json">its</a>{" "}
        <a href="https://github.com/dtolnay/serde-yaml">many</a>{" "}
        <a href="https://github.com/pyfisch/cbor">inte</a>
        <a href="https://github.com/alexcrichton/toml-rs">gra</a>
        <a href="https://github.com/tafia/quick-xml">tions</a>.
    </div>;
    const CREDITS_SCHEMARS = <div>
        This Json Schema was generated through an integration with <a href="https://github.com/GREsau/schemars">Schemars</a>.
    </div>;
    const CREDITS_JSON_TYPEGEN = <div>
        This code was generated through an integration with <a href="https://github.com/evestera/json_typegen">json_typegen</a>.
    </div>;

    switch (props.type) {
        case OutputType.Raw: return CREDITS_SERDE;
        case OutputType.JsonSchema: return CREDITS_SCHEMARS;
        case OutputType.RustTypes: return CREDITS_JSON_TYPEGEN;
        case OutputType.KotlinJacksonTypes: return CREDITS_JSON_TYPEGEN;
        case OutputType.KotlinKotlinxTypes: return CREDITS_JSON_TYPEGEN;
        case OutputType.TypescriptTypes: return CREDITS_JSON_TYPEGEN;
        case OutputType.TypescriptTypeAliasTypes: return CREDITS_JSON_TYPEGEN;
        default:
            console.log(`[Main] unsupported OutputType: ${props.type} ${OutputType[props.type]}`);
            return <></>;
    }
}

function extractOutputTypeText(o: OutputType, output: AllOutputs): string | undefined {
    switch (o) {
        case OutputType.Raw: return output?.rawSchema;
        case OutputType.JsonSchema: return output?.jsonSchema;
        case OutputType.RustTypes: return output?.rust;
        case OutputType.KotlinJacksonTypes: return output?.kotlinJackson;
        case OutputType.KotlinKotlinxTypes: return output?.kotlinKotlinx;
        case OutputType.TypescriptTypes: return output?.typescript;
        case OutputType.TypescriptTypeAliasTypes: return output?.typescriptTypeAlias;
        default:
            console.log(`[Main] unsupported OutputType: ${o} ${OutputType[o]}`);
            return undefined;
    }
}

function outputTypeToLanguage(o: OutputType): string | undefined {
    switch (o) {
        case OutputType.Raw: return "json";
        case OutputType.JsonSchema: return "json";
        case OutputType.RustTypes: return "rust";
        case OutputType.KotlinJacksonTypes: return "kotlin";
        case OutputType.KotlinKotlinxTypes: return "kotlin";
        case OutputType.TypescriptTypes: return "typescript";
        case OutputType.TypescriptTypeAliasTypes: return "typescript";
        default:
            console.log(`[Main] unsupported OutputType: ${o} ${OutputType[o]}`);
            return undefined;
    }
}

function outputTypeToName(o: OutputType): string | undefined {
    switch (o) {
        case OutputType.Raw: return "Raw Schema";
        case OutputType.JsonSchema: return "Json Schema";
        case OutputType.RustTypes: return "Rust";
        case OutputType.KotlinJacksonTypes: return "Kotlin Jackson";
        case OutputType.KotlinKotlinxTypes: return "Kotlin Kotlinx";
        case OutputType.TypescriptTypes: return "Typescript";
        case OutputType.TypescriptTypeAliasTypes: return "Typescript Type Alias";
        default:
            console.log(`[Main] unsupported OutputType: ${o} ${OutputType[o]}`);
            return undefined;
    }
}

function outputTypeFileExtension(o: OutputType): string | undefined {
    switch (o) {
        case OutputType.Raw: return "json";
        case OutputType.JsonSchema: return "json";
        case OutputType.RustTypes: return "rs";
        case OutputType.KotlinJacksonTypes: return "kt";
        case OutputType.KotlinKotlinxTypes: return "kt";
        case OutputType.TypescriptTypes: return "ts";
        case OutputType.TypescriptTypeAliasTypes: return "ts";
        default:
            console.log(`[Main] unsupported OutputType: ${o} ${OutputType[o]}`);
            return undefined;
    }
}