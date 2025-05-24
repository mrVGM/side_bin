const { invoke } = window.__TAURI__.core;

function getWebCurrentWebview() {
    const webViewWindow = window.__TAURI__.webViewWindow;
    const wnd = webViewWindow.getWebCurrentWebviewWindow();
    return wnd;
}

async function resizeWin(x, y, w, h) {
    await invoke("resize_win", { 
        x: x,
        y: y,
        w: w,
        h: h
    });
}

async function monitorCommand(action, file) {
    let response = await invoke("monitor_command", {
        action,
        file
    });

    console.log(response);
    response = JSON.parse(response);

    return response;
}

let fileCallbacks = [];

async function registerFile(elem) {
    const file = elem.storedFile;
    const fileIdObj = await new Promise(async resolve => {
        async function task() {
            let fileId = await monitorCommand("register", file);
            resolve(fileId);
        }
        fileCallbacks.push(task);
    });
    const fileId = fileIdObj.id;
    
    console.log("file registered as", fileId);

    let age = 0;
    while (elem.storedFile) {
        const state = await new Promise(resolve => {
            async function task() {
                let state = await monitorCommand("update", fileId);
                resolve(state);
            }
            fileCallbacks.push(task);
        });
        if (state.Certain) {
            age = 0;
            console.log("Certain");
        }
        else {
            if (age > 3) {
                elem.storedFile = undefined;
            }
            console.log(state, age);
            ++age;
        }
    }
    fileCallbacks.push(async () => {
        await monitorCommand("unregister", fileId);
    });
}

async function mainTick() {
    async function tick() {
        const funcs = fileCallbacks;
        fileCallbacks = [];

        const promises = funcs.map(f => {
            return f();
        });
        await Promise.all(promises);

        await monitorCommand("tick", "");
    }
    while (true) {
        await tick();
        await new Promise(resolve => {
            setTimeout(() => {
                resolve();
            }, 1000);
        });
    }
}

window.addEventListener("DOMContentLoaded", async () => {
    const mainElement = document.querySelector("#main");
    mainTick();

    let state;
    function expandWindow() {
        if (state !== "expanded") {
            resizeWin(-20, 300, 80, 250);
        }
        state = "expanded";
        mainElement.style.display = "";
    }
    function collapseWindow() {
        if (state !== "collapsed")
        {
            resizeWin(-20, 300, 30, 250);
        }
        state = "collapsed";
        mainElement.classList.remove("main-hidden");
        mainElement.style.display = "none";
    }

    let droppedFile = undefined;
    const item1 = document.querySelector("#slot1");
    const item2 = document.querySelector("#slot2");
    const item3 = document.querySelector("#slot3");

    const items = [item1, item2, item3];

    const dropSlots = [];
    const hoverSlots = [];

    items.forEach(item => {
        dropSlots.push((payload) => {
            console.log("drop start");
            const rect = item.getBoundingClientRect();
            const pos = payload.position;
            const paths = payload.paths;
            const file = paths[0];

            if (paths.length > 1) {
                return;
            }
            if (item.storedFile) {
                return;
            }
            if (
                rect.x <= pos.x &&
                pos.x <= rect.x + rect.width &&
                rect.y <= pos.y &&
                pos.y <= rect.y + rect.height) {
                item.storedFile = file;
                registerFile(item);
                item.classList.add("item-full");
                console.log("file stored");
            }
        });

        hoverSlots.push((payload) => {
            if (!payload) {
                item.classList.remove("item-hovered");
                return;
            }
            const rect = item.getBoundingClientRect();
            const pos = payload.position;
            if (
                rect.x <= pos.x &&
                pos.x <= rect.x + rect.width &&
                rect.y <= pos.y &&
                pos.y <= rect.y + rect.height) {
                if (!item.storedFile) {
                    item.classList.add("item-hovered");
                }
            }
            else {
                item.classList.remove("item-hovered");
            }
        });
    });

    items.forEach(item => {
        item.addEventListener("mousedown", () => {
            if (!item.storedFile) {
                return;
            }
            const { startDrag } = window.__TAURI__.drag;
            startDrag({
                item: [item.storedFile],
                icon: "",
                mode: "move"
            });
            item.storedFile = undefined;
        });
    });

    let webview = window.__TAURI__.webview;
    const unlisten = await webview
        .getCurrentWebview()
        .onDragDropEvent((event) => {
        if (event.payload.type === 'over') {
            expandWindow();
            hoverSlots.forEach(slot => {
                slot(event.payload);
            });
        } else if (event.payload.type === 'drop') {
            droppedFile = event.payload.paths[0];
            dropSlots.forEach(slot => {
                slot(event.payload);
            });
            hoverSlots.forEach(slot => {
                slot();
            });
        } else {
            console.log('File drop cancelled');
            collapseWindow();
        }
    });

    ["mouseover"].forEach(name => {
        document.addEventListener(name, (evt) => {
            if (evt.relatedTarget)
            {
                return;
            }
            expandWindow();
        });
    });

    ["mouseout"].forEach(name => {
        document.addEventListener(name, (evt) => {
            if (evt.relatedTarget)
            {
                return;
            }
            collapseWindow();
        });
    });

});

