import React from 'react'
import './App.css'
import ServerControl from './components/ServerControl';

function App() {
  return (
    <div className='App'>
      <h1>Servidor iniciado em:</h1>
      <ServerControl />
    </div>
  );
}

export default App;