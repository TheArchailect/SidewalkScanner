import React from "react"
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter, Routes, Route } from "react-router-dom"
import './index.css'
import Home from './pages/Home.jsx'
import Workspace from "./pages/Workspace.jsx"

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/Workspace" element={<Workspace />} />
      </Routes>
    </BrowserRouter>
  </StrictMode>,
);
