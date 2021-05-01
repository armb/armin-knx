function myFunction(body) {
    let req = new XMLHttpRequest();
    req.open("PUT", document.documentURI);
    req.send(body);
    //document.getElementById("demo").innerText = "BUTTON";
}

function toggleVisibility(id) {
    var e = document.getElementById(id);
    if (e.style.display != "block") {
	e.style.display = "block";
    } else {
	e.style.display = "none";
    }
}

function testWebsocket() {
    if ("WebSocket" in window) {
	alert("WebSocket is available");
    }
}

async function testJsonPost() {
    const response = await fetch('/n/tv/input/key', {
	method: 'POST',
	body: "{ \"key\": \"Pause\"}",
	headers: { 'Content-Type': 'application/json' }
    });
    const myjson = await response.json();
    alert("response: " + myjson);
    console.log(myjson);
}
