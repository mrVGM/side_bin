#pragma once

#include <coroutine>
#include <variant>
#include <queue>

struct Blocker
{
    std::coroutine_handle<> _h;
    bool await_ready() const noexcept;
    void await_suspend(std::coroutine_handle<> handle);

    void await_resume() const noexcept;

    void Unblock();
};

class CoroutinePlayer;
struct Coroutine
{
	struct promise_type
	{
		CoroutinePlayer* _player = nullptr;
		Coroutine get_return_object();
		std::suspend_never initial_suspend() noexcept;
		std::suspend_never final_suspend() noexcept;
		void unhandled_exception();
		void return_void();
	};

	std::coroutine_handle<promise_type> handle;
	Coroutine(std::coroutine_handle<promise_type> h);
};

class CoroutinePlayer
{
private:
	Coroutine _coroutine;
	
public:
	CoroutinePlayer(const Coroutine& coroutine);
	~CoroutinePlayer();
};

