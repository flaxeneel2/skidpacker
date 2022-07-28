window.onload = () => {
    setInterval(updateInfo, 100);
}
function updateInfo() {
    let xmlHttp = new XMLHttpRequest();
    xmlHttp.open( "GET", `http://127.0.0.1:8080/api/data`, false ); // false for synchronous request
    xmlHttp.send( null );
    let ans = JSON.parse(xmlHttp.response)
    document.getElementById("classNum").innerHTML = ans.accepted;
    document.getElementById("jarName").innerHTML = ans.name;
}