#pragma once

#include <string>

std::string toUTF8(const std::wstring& src);
std::wstring fromUTF8(const std::string& src);
