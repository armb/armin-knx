function myFunction(room_id) {
    toggleVisibility("room_actors." + room_id)
}


function toggleVisibility(id) {
    const e = document.getElementById(id);
    if (e.style.display != "block") {
        e.style.display = "block";
    }
// never hide
//    else {
//        e.style.display = "none";
//    }
}


async function testJsonPost() {
    const response = await fetch('/n/tv/input/key', {
        method: 'POST',
        body: "{ \"key\": \"Pause\"}",
        headers: { 'Content-Type': 'application/json' }
    });
    const myjson = await response.json();
    // alert("response: " + myjson);
    console.log(myjson);
    document.getElementById('message').textContent = myjson['eg.flur.brightness'];
}

// https://developer.mozilla.org/en-US/docs/Web/API/setInterval
var nIntervId;
function onLoad() {
    nIntervId = setInterval(onTimerEpired, 1000);
}
function onTimerEpired() {
    updateSensorData();
}
