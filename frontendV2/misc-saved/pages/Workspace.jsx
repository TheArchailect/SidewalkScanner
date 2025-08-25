import BevyViewport from "../components/BevyViewport";

export default function Workspace() {
    return (
        <main style={{ padding: 24}}>
            <div style ={{ width: "100vw", height: "100vh" }}>
                <BevyViewport />
            </div>
            <h2>Workspace</h2>
            <p>LiDAR tools will live here.</p>
        </main>
    );
}