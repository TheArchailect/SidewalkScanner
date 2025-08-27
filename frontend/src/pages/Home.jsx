import { useState } from 'react'
import reactLogo from '../assets/react.svg'
import viteLogo from '/vite.svg'
import '../css/Home.css'
import HomeButton from '../components/HomeButton'


function Home() {
  return (
    <>
      <div>
        <a href="https://ifpedestrians.org" target="_blank">
          <img src="/images/ifpLogo.png" className="logo" alt="IFP logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1 >
        <a classname="SidewalkScannerTitle" href="https://ifpedestrians.org/sidewalkscanner">Sidewalk Scanner</a>
        </h1>
      <div className="card">
        <HomeButton />
        <p>
          Sidewalk Scanner Point Cloud Editor
        </p>
      </div>
      <p className="read-the-docs">
        © ____________________________ 2025
      </p>
    </>
  )
}

export default Home
