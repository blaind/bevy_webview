import React, { Component, useState, useEffect } from 'react';
import logo from './logo.svg';
import './App.css';

function SampleApp() {
  let [cubeCount, setCubeCount] = useState(0);

  let handleCubeCount = (count) => {
    setCubeCount(count);
  };

  if (window.rpc) {
    useEffect(() => {
      window.rpc.on("cube_count", handleCubeCount);
      return () => {
        window.rpc.removeListener("cube_count", handleCubeCount);
      }
    }, []);
  }

  let spawnCubes = (count) => {
    window.rpc.notify("spawn_cubes", { count })
  }

  let destroyCubes = () => {
    window.rpc.notify("destroy_cubes");
  }

  return (
    <div className="App">
      <div className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <h2>bevy_webview &lt;3 React</h2>
      </div>

      <div className="App-intro">
        <p>
          To get started, edit <code>src/App.js</code> and save to reload.
        </p>

        <ul style={{ textAlign: "left" }}>
          <li>Live reload - works!</li>
          <li>Transparency - works!</li>
          <li>Events with Bevy - works!</li>
        </ul>

        <p>Current cube count: {cubeCount}</p>

        <p>
          <button onClick={() => spawnCubes(3)}>Spawn cubes</button>
          {cubeCount > 0 ? <button onClick={() => destroyCubes()}>Destroy cubes</button> : null}
        </p>
      </div>
    </div >
  )
}

class App extends Component {
  render() {
    return (
      <SampleApp />
    );
  }
}

export default App;
