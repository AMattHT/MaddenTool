import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { getAppInfo } from "./api/app-info";
import App from "./App";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("Missing #root element");

createRoot(root).render(
  <StrictMode>
    <App loadAppInfo={getAppInfo} />
  </StrictMode>,
);
