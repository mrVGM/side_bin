#include "crt.h"

bool Blocker::await_ready() const noexcept
{
	return false;
}

void Blocker::await_suspend(std::coroutine_handle<> handle)
{
	_h = handle;
}

void Blocker::await_resume() const noexcept
{
}

void Blocker::Unblock()
{
	_h.resume();
}

Coroutine Coroutine::promise_type::get_return_object()
{
	return Coroutine(std::coroutine_handle<promise_type>::from_promise(*this));
}

std::suspend_never Coroutine::promise_type::initial_suspend() noexcept
{
	return {};
}
std::suspend_never Coroutine::promise_type::final_suspend() noexcept
{ 
	return {};
}

void Coroutine::promise_type::return_void()
{
}

void Coroutine::promise_type::unhandled_exception()
{
}

Coroutine::Coroutine(std::coroutine_handle<promise_type> h) :
	handle(h)
{
}

CoroutinePlayer::CoroutinePlayer(const Coroutine& coroutine) :
	_coroutine(coroutine)
{
	coroutine.handle.promise()._player = this;
	coroutine.handle.resume();
}

CoroutinePlayer::~CoroutinePlayer()
{
	_coroutine.handle.destroy();
}

