import { useState } from 'react'
import reactLogo from '../assets/react.svg'
import viteLogo from '/vite.svg'
import '../Home.css'

// Workspace Import
import ButtonEvents from "../components/buttonEvents.jsx";


function Home() {
  const [count, setCount] = useState(0)

  return (
    <>
      <div>
        <a href="https://ifpedestrians.org/" target="_blank">
          <img src="/ifp.jpg" className="logo" alt="ifp logo" />
        </a>
        
      </div>
      <h1>
        <a href="https://ifpedestrians.org/sidewalkscanner/"
          target="_blank"
          rel="noopener noreferrer"
          className="heading-link">
          Sidewalk Scanner
        </a>
      </h1>
      <div className="card">
        <ButtonEvents />
        <p>
          
        </p>
      </div>
      <p className="read-the-docs">
        ©______________________
      </p>
    </>
  )
}

export default Home
