#include "fs_monitor.h"
#include "monitors.h"

#include <queue>
#include <Windows.h>
#include <variant>
#include <string>
#include <iostream>
#include <map>

namespace
{
    std::string _lastError;
    std::queue<Blocker*> _scheduler;
	std::map<std::string, DirMonitor*> _monitors;
}

std::string toUTF8(const std::wstring& src)
{
	int num = WideCharToMultiByte(
		CP_UTF8,
		0,
		src.c_str(),
		src.size(),
		nullptr,
		0,
		nullptr,
		nullptr);

	std::string res(num, 0);
	WideCharToMultiByte(
		CP_UTF8,
		0,
		src.c_str(),
		src.size(),
		const_cast<char*>(res.c_str()),
		num,
		nullptr,
		nullptr);

	return res;
}

std::wstring fromUTF8(const std::string& src)
{
	int num = MultiByteToWideChar(
		CP_UTF8,
		0,
		src.c_str(),
		src.size(),
		nullptr,
		0
	);

	std::wstring res(num, 0);
	MultiByteToWideChar(
		CP_UTF8,
		0,
		src.c_str(),
		src.size(),
		const_cast<wchar_t*>(res.c_str()),
		num);

	return res;
}


bool Boot(const char* dir)
{
    std::cout << "Boot" << std::endl;
    std::wstring dirWide = fromUTF8(dir);

    bool res = true;
    try
    {
		DirMonitor* newMonitor = new DirMonitor(dirWide.c_str(), _scheduler);
		_monitors[dir] = newMonitor;
    }
    catch (std::string err)
    {
        _lastError = err;
        res = false;
    }
    return res;
}

bool Tick(const char* dir)
{
	bool res = true;
	try
	{
		auto it = _monitors.find(dir);
		if (it == _monitors.end())
		{
			throw "Dir Monitor not found!";
		}
		DirMonitor* monitor = it->second;
		monitor->Tick();
	}
	catch (std::string err)
	{
		_lastError = err;
		res = false;
	}
    return res;
}

const FileSystemEvent* Peek(const char* dir)
{
	auto it = _monitors.find(dir);
	if (it == _monitors.end())
	{
		return nullptr;
	}
	DirMonitor* monitor = it->second;

    std::queue<FileSystemEvent>& events = monitor->GetFSEvents();
    if (events.empty())
    {
        return nullptr;
    }

    return &events.front();
}

void Pop(const char* dir)
{
	auto it = _monitors.find(dir);
	if (it == _monitors.end())
	{
		return;
	}
	DirMonitor* monitor = it->second;

    std::queue<FileSystemEvent>& events = monitor->GetFSEvents();
    if (!events.empty())
    {
        events.pop();
    }
}

void Shutdown(const char* dir)
{
	DirMonitor* monitor = _monitors[dir];
	delete monitor;
    _monitors.erase(dir);
}

const char* GetLastErr(int* len)
{
    *len = _lastError.size();
    return _lastError.c_str();
}

int GetAction(const FileSystemEvent* event)
{
    return event->_action;
}

const char* GetFile(const FileSystemEvent* event, int* size)
{
    *size = event->file.size();
    return event->file.c_str();
}


