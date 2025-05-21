#include "monitors.h"
#include "util.h"

#include <iostream>

namespace
{
    std::string GetLastErrorAsString()
	{
		//Get the error message ID, if any.
		DWORD errorMessageID = ::GetLastError();
		if(errorMessageID == 0) {
			return std::string(); //No error message has been recorded
		}
		
		LPSTR messageBuffer = nullptr;

		//Ask Win32 to give us the string version of that message ID.
		//The parameters we pass in, tell Win32 to create the buffer that holds the message for us (because we don't yet know how long the message string will be).
		size_t size = FormatMessageA(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
									 NULL, errorMessageID, MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), (LPSTR)&messageBuffer, 0, NULL);
		
		//Copy the error message into a std::string.
		std::string message(messageBuffer, size);
		
		//Free the Win32's string's buffer.
		LocalFree(messageBuffer);
				
		return message;
	}
}

#define THROW_ERR \
{ \
	throw GetLastErrorAsString(); \
} \

#define THROW_CRT_ERR \
{ \
    _thrownError = true; \
    _error = GetLastErrorAsString(); \
    co_await _blocker; \
} \


Coroutine DirMonitor::StartMonitoring()
{
    const int BUFFER_SIZE = 1024;
    static BYTE buffer[BUFFER_SIZE];
    DWORD bytesReturned;
    OVERLAPPED overlapped = {0};
    overlapped.hEvent = _hEvent;

    while (true)
    {
        ZeroMemory(&buffer, sizeof(buffer));
        ResetEvent(overlapped.hEvent);

        BOOL result = ReadDirectoryChangesW(
            _hDir,
            buffer,
            sizeof(buffer),
            TRUE, // Watch subdirectories
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_DIR_NAME,
//          FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_DIR_NAME |
//          FILE_NOTIFY_CHANGE_ATTRIBUTES | FILE_NOTIFY_CHANGE_SIZE |
//          FILE_NOTIFY_CHANGE_LAST_WRITE,
            NULL, // Not used with overlapped I/O
            &overlapped,
            NULL  // Completion routine, NULL if polling or waiting on event
        );

        if (!result) {
            THROW_CRT_ERR
        }

        DWORD waitRes;
        // Wait for the overlapped I/O to complete
        while ((waitRes = WaitForSingleObject(overlapped.hEvent, 0)) == WAIT_TIMEOUT)
        {
            _scheduler.push(&_blocker);
            co_await _blocker;
        }
        if (waitRes != WAIT_OBJECT_0)
        {
            THROW_CRT_ERR
        }

        if (!GetOverlappedResult(_hDir, &overlapped, &bytesReturned, FALSE)) {
            THROW_CRT_ERR
        }

        DWORD offset = 0;
        FILE_NOTIFY_INFORMATION* pNotify;

        do {
            pNotify = (FILE_NOTIFY_INFORMATION*)((BYTE*)buffer + offset);
            FileSystemEvent& evt = _fsEvents.emplace(FileSystemEvent{ static_cast<int>(pNotify->Action) });

            static wchar_t buff[MAX_PATH] = {};

            wcsncpy_s(
                buff,
                _countof(buff),
                pNotify->FileName,
                pNotify->FileNameLength / sizeof(WCHAR));
            evt.file = toUTF8(buff);

            offset += pNotify->NextEntryOffset;
        } while (pNotify->NextEntryOffset != 0);
    }
}

DirMonitor::DirMonitor(const wchar_t* dir, std::queue<Blocker*>& scheduler) :
    _scheduler(scheduler)
{
	_hDir = CreateFileW(
        dir,
        FILE_LIST_DIRECTORY,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        NULL,
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED,
        NULL
    );
    if (_hDir == INVALID_HANDLE_VALUE) {
        THROW_ERR
    }
    _hEvent = CreateEvent(NULL, TRUE, FALSE, NULL);
    if (!_hEvent) {
        THROW_ERR
    }

    _player.emplace<CoroutinePlayer>(StartMonitoring());
}

DirMonitor::~DirMonitor()
{
    CloseHandle(_hDir);
    CloseHandle(_hEvent);
}

void DirMonitor::Tick()
{
    if (!_scheduler.empty())
    {
        Blocker* b = _scheduler.front();
        _scheduler.pop();
        b->Unblock();
    }

    if (_thrownError)
    {
        throw _error;
    }
}

std::queue<FileSystemEvent>& DirMonitor::GetFSEvents()
{
    return _fsEvents;
}


