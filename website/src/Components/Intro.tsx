import React, { useState } from 'react';

import { DataType, DATA_TYPES } from '../definitions';

export function Intro(props: {
  onFileLoad: (files: FileList, dataType: DataType) => void,
  onUrlSelection: (file: string, dataType: DataType) => void
}) {
  const [files, setFiles] = useState<FileList | null>();
  const [text, setText] = useState<string | undefined>();
  const [dataType, setDataType] = useState<DataType | undefined>();

  function enumToButton(t: DataType) {
    const typeName = DataType[t].toLowerCase();
    return <button
      key={typeName}
      onClick={(_) => setDataType(t)}
      className={t === dataType ? "active" : ""}
    >{typeName}</button>
  }

  return (
    <div className="welcome">
      <h1>Welcome</h1>
      <p>What do you wish to analyze?</p>
      <div> {DATA_TYPES.map(enumToButton)} </div>

      {dataType !== undefined ? <> <p>Where is it located?</p>
        <div>
          {!files ? <>
            <p><input
              type="text"
              placeholder="Here, conveniently pasted..."
              onChange={(e) => setText(e?.currentTarget.value)} /></p>

            {text ?
              <p><button onClick={() => props.onUrlSelection(textToUrl(text), dataType)}>Confirm</button></p>
              : <></>}
          </>
            : <></>}
        </div>

        <div>
          {!text ? <>
            <p><FileSelectorInput unselected={!files} onFileLoad={setFiles} /></p>

            {files?.length ?
              <button onClick={() => props.onFileLoad(files, dataType)}>Confirm</button>
              : <></>}
          </>
            : <></>}
        </div>
      </> : <></>
      }


      <p>This program works entirely in your browser.</p>

      <button
        className="example"
        onClick={() => props.onUrlSelection("/test.json", DataType.Json)}>Example</button>
    </div>
  );
}


function FileSelectorInput(props: {
  unselected: boolean,
  onFileLoad: (files: (FileList | null)) => void,
}) {
  return (
    <label title="File Import" className="input_file_label">
      <input
        type="file"
        name="file import"
        size={0.1}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => props.onFileLoad(e.target.files)} />
      {props.unselected ? "...or a local file?" : "change file"}
    </label>
  )
}

function textToUrl(text: string): string {
  return URL.createObjectURL(new Blob([text]));
}
