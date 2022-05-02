class Oster {
    constructor(name, description = "", title = "", command = "") {
        this.name = name;
        this.description = description;
        this.title = title;
        this.command = command;
        this.render = () => {
            let template = `<div class="card">
                <p class="card-title"></p>
                <p class="card-desc"></p>
                <div class="card-cmd">
                    <span class="card-cmd-text"></span>
                </div>
                <div class="card-btn">
                    <button class="button">Run</button>
                </div>
            </div>`;
            let parser = new DOMParser();
            let templateparsed = parser.parseFromString(template, "text/html");
            templateparsed.querySelector(".card-title").innerText = this.title;
            templateparsed.querySelector(".card-desc").innerText = this.description;
            templateparsed.querySelector(".card-cmd-text").innerText = this.command;
            templateparsed.querySelector(".card-btn .button").onclick = this.callback;
            return templateparsed.documentElement;
        }
        this.callback = () => {
            let url = new URL("/command", `http://${settings.ip}:${settings.port}`);
            let urlqueries = new URLSearchParams();
            urlqueries.append("password", settings.password);
            urlqueries.append("cmd", this.command);
            url.search = urlqueries.toString();
            let request = new Request(url, { method: "get", mode: "no-cors" });
            fetch(request)
                .then(res => {
                    switch (res.status) {
                        case 200:
                            alert(res.statusText);
                            break;
                        case 401:
                            alert("password is wrong");
                            break;
                        case 500:
                            alert("command not found");
                            break;
                        default:
                            break;
                    }
                })
                .catch((err) => {
                    console.error(err);
                })
        }
    }
    setName(name) {
        this.name = name;
    }
    setDescription(desc) {
        this.description = desc;
    }
    setTitle(t) {
        this.title = t;
    }
    setCommand(cmd) {
        this.command = cmd;
    }
}
class Osters {
    constructor(osters = []) {
        if (!Array.isArray(osters)) {
            throw Error("osters is not an array");
        }
        this.osters = osters;

    }
    drop() {
        this.osters = [];
        this.#updateStorage();
    }
    fill(osters) {
        if (!Array.isArray(osters)) {
            throw Error("osters is not an array");
        }
        this.osters = osters;
        this.#updateStorage();
    }
    add(oster) {
        if (!oster instanceof Oster) {
            throw Error("oster is not an instance of Oster")
        }
        this.osters.push(oster);
        this.#updateStorage();
    }
    #updateStorage() {
        window.localStorage.setItem("osters", JSON.stringify(osters));
    }
    grabFromStorage() {
        var tmp = window.localStorage.getItem("osters");
        if (tmp !== null && tmp) {
            JSON.parse(tmp)["osters"].map((oster) => {
                this.osters.push(new Oster(oster.name, oster.description, oster.title, oster.command))
            });
            console.log(this.osters);
        }
    }
    iterator() {
        return this.osters;
    }
}

let settings = {};

let osters = new Osters();
osters.grabFromStorage();


function is_ready() {
    const st = window.localStorage;
    if (st.getItem("settings") !== null) {
        settings = JSON.parse(st.getItem("settings"));
        return true;
    }
    return false;
}
window.addEventListener("DOMContentLoaded", function () {

    if (is_ready()) {
        this.cards = this.document.querySelector(".cards");
        this.addsetting = this.document.querySelector(".popup");
        document.querySelector(".add").addEventListener("click", () => {
            trigger_add();
            document.querySelector(".body").classList.add("nointeractive")
        })
        function trigger_add() {
            this.addsetting.style.visibility = "visible";
        } function untrigger_add() {
            this.addsetting.style.visibility = "hidden";
        }

        if (osters.iterator().length > 0) {

            [...osters.iterator()].map((oster) => {
                this.cards.append(oster.render());
            })
        } else {
            this.cards.innerText = "No oster is found..., try adding one"
        }

    } else {
        window.location.pathname = "settings.html";
    }

    document.querySelector(".form").addEventListener("submit", (e) => {
        e.preventDefault();
        e.preventDefault();
        const form = new FormData(document.querySelector(".form"));
        const name = form.get("name");
        const title = form.get("title");
        const command = form.get("cmd");
        const description = form.get("desc");
        osters.add(new Oster(name, description, title, command))
        untrigger_add();
        document.querySelector(".body").classList.remove("nointeractive");
        window.location.reload();
    })
});