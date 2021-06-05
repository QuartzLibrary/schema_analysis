import React, { useState } from 'react';

import { Result } from './Components/Result';
import { Intro } from './Components/Intro';
import { DataType } from './definitions';

enum AppStatus {
  Welcome,
  Display,
}


function App() {
  const [status, setStatus] = useState(AppStatus.Welcome);
  const [dataUrls, setDataUrls] = useState<string[]>([]);
  const [dataType, setDataType] = useState<DataType>();

  return (
    <div>
      {(AppStatus.Welcome === status) ?
        <Intro onFileLoad={handleFileSelection} onUrlSelection={handleUrl} /> : <></>}

      {(AppStatus.Display === status && dataType !== undefined) ?
        <Result dataUrls={dataUrls} dataType={dataType} /> : <></>}

      <a href="https://github.com/QuartzLibrary/schema_analysis">
        <button className="code-button"><span role="img" aria-label="Code">
          <span className="cb-normal">{"<'de, T>"}</span>
          <span className="cb-hover">&nbsp;&nbsp;code&nbsp;&nbsp;</span>
        </span></button>
      </a>

      <p className="pr-call">
        Don't see the format you were looking for? <br />
        It might be just a <a href="https://github.com/QuartzLibrary/schema_analysis">PR</a> away.
      </p>
    </div>
  );


  async function handleFileSelection(files: (FileList | null), dataType: DataType) {
    if (!files) return;

    // Revoke any previously assigned URLs.
    dataUrls.map((url, i) => URL.revokeObjectURL(url));

    let newFileURLs = new Array<string>(files.length);
    for (let i = 0; i < newFileURLs.length; i++) {
      newFileURLs[i] = URL.createObjectURL(files[i]);
    }

    setDataUrls(newFileURLs);
    setDataType(dataType);
    setStatus(AppStatus.Display);
  }

  function handleUrl(remoteUrl: string, dataType: DataType) {
    setDataUrls([remoteUrl]);
    setDataType(dataType);
    setStatus(AppStatus.Display);
  }
}


export default App;