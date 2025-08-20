import { useNavigate } from "react-router-dom";

export default function ButtonsEvent() {
  const navigate = useNavigate();
  return <button onClick={() => navigate("/Workspace")}>Begin</button>;
}
