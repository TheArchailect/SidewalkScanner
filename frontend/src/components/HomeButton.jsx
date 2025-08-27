export default function homeRedirect() {
    function clickBegin() {
        window.location.href = "/renderer/SidewalkScanner.html";
    }

    return <button onClick={clickBegin}>Begin</button>
}