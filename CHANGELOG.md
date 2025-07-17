# 0.2.1 (2025-07-17)
- Add ANSI color support for log messages
- `CallParam::typename()` added to show concrete, human-readable type information when the types of condition and data are not identical.


# 0.2.0 (2025-07-16)
### 🆕 New Features
- Added logging support with detailed debug information to assist troubleshooting
### ⚠️ Breaking Changes
- Deprecated `task()` in favor of `into_task()` to follow Rust's ownership conventions


# 0.1.1 (2025-7-10)
- `Courier<F,C,R>`: The generic parameter `C` refers to the element type inside it, and is no longer wrapped in `Option`. Changing from `Courier<F, (Option<P>,..), R>`  to `Courier<F, (P,..), R>`.
- Examples: Improved clarity and readability


# 0.1.0 (2025-7-9)
- Initial implementation of core functionality.
- Implemented basic concepts of task, queue, thread and pool.