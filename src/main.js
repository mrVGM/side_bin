const { invoke } = window.__TAURI__.core;

function createDOMElement(html) {
    let tmp = document.createElement('template');
    tmp.innerHTML = html;
    let elem;
    tmp.content.childNodes.forEach(x => {
        if (x.nodeName !== "#text") {
            elem = x;
        }
    });
    return elem;
}

function getWebCurrentWebview() {
    const webViewWindow = window.__TAURI__.webViewWindow;
    const wnd = webViewWindow.getWebCurrentWebviewWindow();
    return wnd;
}

async function exitApp() {
    await invoke("exit_app", { });
}

async function openFileDir(file) {
    await invoke("open_file_directory", {
        file
    });
}

async function readConfig() {
    const config = await invoke("read_config", { });
    console.log(config);
    let configJSON = {};
    try {
        configJSON = JSON.parse(config);
    }
    catch(_) { }
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

async function getIcon(file) {
    let response = await invoke("get_file_icon", {
        file
    });

    response = JSON.parse(response);

    return response;
}

const droppedFiles = {};
let fileCallbacks = [];

async function registerFile(elem) {
    const overlay = elem.querySelector("#overlay");
    const name = elem.querySelector("#name");
    const itemIcon = elem.querySelector("#item-icon");

    const file = elem.storedFile;
    const icon = await getIcon(file);

    if (icon.valid) {
        const imageData = new Uint8Array(icon.data);
        const image = new Blob(
            [imageData],
            {
                type: 'image/png'
            }
        );
        const imageUrl = URL.createObjectURL(image);
        itemIcon.style.backgroundImage = `url('${imageUrl}')`;
    }

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

    async function checkFileTag() {
        const newestTag = await new Promise(resolve => {
            async function task() {
                if (!elem.storedFile) {
                    resolve();
                    return;
                }
                let resp = await getFileTag(elem.storedFile);
                if (resp.valid) {
                    resolve(resp.tag);
                    return;
                }
                resolve();
            }
            fileCallbacks.push(task);
        });

        return newestTag === fileId;
    }

    const tagCheckOk = Symbol("checked");
    const tagCheckInProgress = Symbol("in progress");

    let tagCheck = tagCheckOk;

    let stopChecks;
    let tagCheckRoutine = new Promise(resolve => {
        let goOn = true;
        stopChecks = () => {
            goOn = false;
        };

        function invalidate() {
            tagCheck = tagCheckInProgress;
            if (goOn) {
                setTimeout(invalidate, 1000);
            }
            else {
                resolve();
            }
        }

        setTimeout(invalidate, 1000);
    });

    let age = 0;
    while (!stop && elem.storedFile) {
        const state = await new Promise(resolve => {
            async function task() {
                let state = await monitorCommand("update", fileId);
                resolve(state);
            }
            fileCallbacks.push(task);
        });

        if (tagCheck === tagCheckInProgress) {
            const ok = await checkFileTag();
            if (ok) {
                tagCheck = tagCheckOk;
            }
        }

        if (state.Certain && tagCheck === tagCheckOk) {
            age = 0;
            elem.storedFile = state.Certain.path;
            const lastSlash = elem.storedFile.lastIndexOf("\\");
            if (lastSlash >= 0)
            {
                let fileName = elem.storedFile.substring(lastSlash + 1);
                if (fileName.length > 30) {
                    fileName = fileName.substring(fileName.length - 30);
                    fileName = "..." + fileName;
                }
                name.innerHTML = fileName;
            }
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
    overlay.style.display = "none"
    elem.style.backgroundImage = "";

    stopChecks();
    await tagCheckRoutine;
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

async function updateConfig() {
    let config = await readConfig();
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

    return config;
}

window.addEventListener("DOMContentLoaded", async () => {
    await setupTray();

    let config = await updateConfig();
    async function refreshConfig() {
        config = await updateConfig();
        setTimeout(refreshConfig, 1000);
    }
    setTimeout(refreshConfig, 1000);

    const mainElement = document.querySelector("#main");
    if (config.alignment === "vertical") {
        mainElement.classList.add("container-vertical");
    }
    
    mainTick();

    let state;
    async function expandWindow() {
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
            await resizeWin(
                position[0],
                position[1],
                size[0],
                size[1]);
        }
        state = "expanded";
        mainElement.style.display = "";
    }
    async function collapseWindow() {
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
            await resizeWin(
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

    const container = document.querySelector("#main");
    const items = [];
    for (let i = 0; i < 3; ++i)
    {
        let item = createDOMElement(`
<div class="item">
    <div class="slot-overlay" id="overlay">
        <div class="item-icon" id="item-icon"></div>
        <div class="name" id="name"></div>
        <div class="close" id="close"></div>
    </div>
</div>
        `);
        container.appendChild(item);
        items.push(item);
        const close = item.querySelector("#close");
        item.close = close;
    }

    let droppedFile = undefined;

    const dropSlots = [];
    const hoverSlots = [];

    items.forEach(item => {
        let overlay = item.querySelector("#overlay");
        overlay.style.display = "none";
        item.closeFunc = undefined;
        item.close.addEventListener("click", () => {
            if (item.closeFunc) {
                item.closeFunc();
            }
            item.closeFunc = undefined;
        });

        item.addEventListener("dblclick", () => {
            if (item.storedFile) {
                openFileDir(item.storedFile);
            }
        });

        function hoverSlot(hovered) {
            item.classList.remove("item-hovered");
            if (hovered) {
                item.classList.add("item-hovered");
            }
        }

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
            if (file.startsWith("\\\\")) {
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
                overlay.style.display = "";
            }
        });


        hoverSlots.push((payload) => {
            if (!payload) {
                hoverSlot(false);
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
                    hoverSlot(true);
                }
            }
            else {
                hoverSlot(false);
            }
        });
    });

    items.forEach(item => {
        item.addEventListener("mousedown", async () => {
            if (!item.storedFile) {
                return;
            }
            let mouseupHandler, mousemoveHandler;
            let shouldDrag = await new Promise(resolve => {
                mouseupHandler = () => {
                    resolve(false);
                };
                mousemoveHandler = () => {
                    resolve(true);
                };
                item.addEventListener("mouseup", mouseupHandler);
                item.addEventListener("mousemove", mousemoveHandler);
                setTimeout(() => {
                    resolve(true);
                }, 1000);
            });
            item.removeEventListener("mouseup", mouseupHandler);
            item.removeEventListener("mousemove", mousemoveHandler);

            if (shouldDrag) {
                const { startDrag } = window.__TAURI__.drag;
                startDrag({
                    item: [item.storedFile],
                    icon: "",
                    mode: "move"
                });
            }
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
    document.addEventListener("mouseover", (evt) => {
        if (evt.relatedTarget)
        {
            return;
        }
        expandWindow();
    });
    document.addEventListener("mouseout", (evt) => {
        if (evt.relatedTarget)
        {
            return;
        }
        collapseWindow();
    });
});

