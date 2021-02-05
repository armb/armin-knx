let source = new EventSource("demo_sse.php");
source.onmessage = function(event) {
    document.getElementById("result").innerHTML += event.data + "<br>";
};

function log(s)
{

    document.getElementById("message").innerHTML = s;
}
