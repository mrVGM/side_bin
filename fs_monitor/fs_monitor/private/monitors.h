#pragma once

#include "fs_monitor.h"
#include "crt.h"

#include <Windows.h>
#include <queue>

class DirMonitor
{
private:
    HANDLE _hDir = INVALID_HANDLE_VALUE;
    HANDLE _hEvent = INVALID_HANDLE_VALUE;

    std::queue<FileSystemEvent> _fsEvents;

    std::queue<Blocker*>& _scheduler;

    Blocker _blocker;
    std::variant<void*, CoroutinePlayer> _player;
    Coroutine StartMonitoring();

    bool _thrownError = false;
    std::string _error;

public:
    DirMonitor(const wchar_t* dir, std::queue<Blocker*>& scheduler);
    ~DirMonitor();

    void Tick();
    std::queue<FileSystemEvent>& GetFSEvents();
};
