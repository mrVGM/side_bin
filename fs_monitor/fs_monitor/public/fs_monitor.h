#pragma once

#include <Windows.h>
#include <string>

extern "C"
{
	struct FileSystemEvent
	{
		int _action;
		std::string file;
	};

    DWORD __stdcall GetRunningInstance();

	const char* __stdcall GetLastErr(int* len);
	int __stdcall GetAction(const FileSystemEvent* event);
	const char* __stdcall GetFile(const FileSystemEvent* event, int* size);

    bool __stdcall Boot(const char* dir);
    bool __stdcall Tick(const char* dir);
	const FileSystemEvent* __stdcall Peek(const char* dir);
	void __stdcall Pop(const char* dir);
	void __stdcall Shutdown(const char* dir);
}

