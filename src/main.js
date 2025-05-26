const { invoke } = window.__TAURI__.core;
console.log(window.__TAURI__);

function getWebCurrentWebview() {
    const webViewWindow = window.__TAURI__.webViewWindow;
    const wnd = webViewWindow.getWebCurrentWebviewWindow();
    return wnd;
}

async function exitApp() {
    await invoke("exit_app", { });
}

async function readConfig() {
    const config = await invoke("read_config", { });
    const configJSON = JSON.parse(config);
    return configJSON;
}

async function getFileTag(file) {
    let response = await invoke("get_file_tag", {
        file: file
    });

    let responseJSON = JSON.parse(response);
    return responseJSON;
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

    response = JSON.parse(response);

    return response;
}

const droppedFiles = {};
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
    droppedFiles[fileId] = true;

    let stop = false;
    elem.closeFunc = () => {
        stop = true;
    };

    let age = 0;
    while (!stop && elem.storedFile) {
        const state = await new Promise(resolve => {
            async function task() {
                let state = await monitorCommand("update", fileId);
                resolve(state);
            }
            fileCallbacks.push(task);
        });
        if (state.Certain) {
            age = 0;
            elem.storedFile = state.Certain.path;
        }
        else {
            if (age > 3) {
                elem.storedFile = undefined;
            }
            ++age;
        }
    }

    elem.storedFile = undefined;
    await unregister(fileId);
    elem.classList.remove("item-full");
    elem.close.style.display = "none"
}

async function unregister(id) {
    let res = await new Promise(resolve => {
        fileCallbacks.push(async () => {
            const response = await monitorCommand("unregister", id);
            resolve(response);
        });
    });
    delete droppedFiles[id];

    return res;
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
            }, 10);
        });
    }
}

async function setupTray() {
    const { TrayIcon } = window.__TAURI__.tray;
    const { defaultWindowIcon } = window.__TAURI__.app;
    const { Menu } = window.__TAURI__.menu;

    const menu = await Menu.new({
        items: [
            {
                id: 'quit',
                text: 'Quit',
                action: () => {
                    exitApp();
                }
            }
        ]
    });

    let options = {
        menu,
        menuOnLeftClick: true,
        icon: await defaultWindowIcon(),
    };
    
    await TrayIcon.new(options);
}

window.addEventListener("DOMContentLoaded", async () => {
    await setupTray();

    const config = await readConfig();
    console.log(config);

    if (!config.alignment) {
        config.alignment = "vertical";
    }
    if (!config.anchor) {
        config.anchor = [0, 0];
    }
    if (!config.position) {
        config.position = [0, 0];
    }
    if (!config.expanded) {
        config.expanded = [100, 300];
    }
    if (!config.collapsed) {
        config.collapsed = [20, 20];
    }

    const mainElement = document.querySelector("#main");
    if (config.alignment === "vertical") {
        mainElement.classList.add("container-vertical");
    }
    
    mainTick();

    let state;
    function expandWindow() {
        if (state !== "expanded") {
            const size = [config.expanded[0], config.expanded[1]];
            const offset = [
                -config.anchor[0] * size[0],
                -config.anchor[1] * size[1]
            ];
            const position = [
                config.position[0] + offset[0],
                config.position[1] + offset[1]
            ];
            resizeWin(
                position[0],
                position[1],
                size[0],
                size[1]);
        }
        state = "expanded";
        mainElement.style.display = "";
    }
    function collapseWindow() {
        if (state !== "collapsed")
        {
            const size = [config.collapsed[0], config.collapsed[1]];
            const offset = [
                -config.anchor[0] * size[0],
                -config.anchor[1] * size[1]
            ];
            const position = [
                config.position[0] + offset[0],
                config.position[1] + offset[1]
            ];
            resizeWin(
                position[0],
                position[1],
                size[0],
                size[1]);
        }
        state = "collapsed";
        mainElement.classList.remove("main-hidden");
        mainElement.style.display = "none";
    }

    collapseWindow();

    let droppedFile = undefined;
    const item1 = document.querySelector("#slot1");
    const item2 = document.querySelector("#slot2");
    const item3 = document.querySelector("#slot3");

    const close1 = document.querySelector("#close1");
    const close2 = document.querySelector("#close2");
    const close3 = document.querySelector("#close3");

    const items = [item1, item2, item3];
    item1.close = close1;
    item2.close = close2;
    item3.close = close3;

    const dropSlots = [];
    const hoverSlots = [];

    items.forEach(item => {
        item.close.style.display = "none";
        item.closeFunc = undefined;
        item.close.addEventListener("mousedown", () => {
            if (item.closeFunc) {
                item.closeFunc();
            }
            item.closeFunc = undefined;
        });

        dropSlots.push(async payload => {
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

            let resp = await getFileTag(file);
            if (resp.valid && droppedFiles[resp.tag]) {
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
                item.close.style.display = "";
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
            async function checkFileAndDrop() {
                console.log("drop");
                let response = await getFileTag(droppedFile);
                if (response.valid) {
                    const tag = response.tag;
                    console.log("tag", tag);
                }
                dropSlots.forEach(slot => {
                    slot(event.payload);
                });
                hoverSlots.forEach(slot => {
                    slot();
                });
            }
            checkFileAndDrop();
        } else {
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

