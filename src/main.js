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
    await invoke("monitor_command", {
        action,
        file
    });
}

let fileCallbacks = [];

async function registerFile(elem) {
    const file = elem.storedFile;
    await new Promise(async resolve => {
        async function task() {
            await monitorCommand("register", file);
            resolve();
        }
        fileCallbacks.push(task);
    });

    while (elem.storedFile) {
        await new Promise(resolve => {
            async function task() {
                await monitorCommand("update", file);
                resolve();
            }
            fileCallbacks.push(task);
        });
    }
    fileCallbacks.push(async () => {
        await monitorCommand("unregister", file);
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

