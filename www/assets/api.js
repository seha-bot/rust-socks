fetch(window.location.origin + "/help")
.then((response) => response.json())
.then(function (json) {
    let box = document.getElementById("api");

    for (e of json.endpoints) {
        let handler = document.createElement("div");
        handler.setAttribute("class", "handler");
        handler.innerHTML = e.handler;

        let content = document.createElement("div");
        content.setAttribute("class", "content");
        content.appendChild(handler);

        let operation = document.createElement("div");
        operation.setAttribute("class", "operation " + e.operation);
        operation.innerHTML = e.operation;

        let url = document.createElement("div");
        url.setAttribute("class", "url");
        url.innerHTML = e.url;

        let header = document.createElement("div");
        header.setAttribute("class", "holder");
        header.appendChild(operation);
        header.appendChild(url);

        header.addEventListener("click", function() {
            header.classList.toggle("open");
            if (content.style.maxHeight) {
                content.style.maxHeight = null;
                return;
            }
            content.style.maxHeight = content.scrollHeight + "px";
        });

        box.appendChild(header);
        box.appendChild(content);
    }
});