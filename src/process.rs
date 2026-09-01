use std::ffi::{OsStr, OsString};
use std::path::Path;

use crate::error::Result;

#[cfg(windows)]
pub fn run_native(executable: &Path, args: &[OsString]) -> Result<i32> {
    windows::run_native(executable, args)
}

#[cfg(not(windows))]
pub fn run_native(executable: &Path, args: &[OsString]) -> Result<i32> {
    let status = std::process::Command::new(executable)
        .args(args)
        .status()
        .map_err(|source| crate::CodexCliEditorError::io(executable, source))?;
    Ok(status.code().unwrap_or(1))
}

fn quote_windows_argument(argument: &OsStr) -> Result<Vec<u16>> {
    #[cfg(not(windows))]
    use std::os::unix::ffi::OsStrExt;
    #[cfg(windows)]
    use std::os::windows::ffi::OsStrExt;

    #[cfg(windows)]
    let units: Vec<u16> = argument.encode_wide().collect();
    #[cfg(not(windows))]
    let units: Vec<u16> = String::from_utf8_lossy(argument.as_bytes())
        .encode_utf16()
        .collect();

    if units.contains(&0) {
        return Err(crate::CodexCliEditorError::ArgumentNul);
    }
    let needs_quotes = units.is_empty()
        || units
            .iter()
            .any(|unit| matches!(*unit, 0x20 | 0x09 | 0x0a | 0x0b | 0x22));
    if !needs_quotes {
        return Ok(units);
    }

    let mut output = vec![0x22];
    let mut backslashes = 0usize;
    for unit in units {
        if unit == 0x5c {
            backslashes += 1;
        } else if unit == 0x22 {
            output.extend(std::iter::repeat_n(0x5c, backslashes * 2 + 1));
            output.push(unit);
            backslashes = 0;
        } else {
            output.extend(std::iter::repeat_n(0x5c, backslashes));
            output.push(unit);
            backslashes = 0;
        }
    }
    output.extend(std::iter::repeat_n(0x5c, backslashes * 2));
    output.push(0x22);
    Ok(output)
}

fn build_windows_command_line(executable: &Path, args: &[OsString]) -> Result<Vec<u16>> {
    let mut output = quote_windows_argument(executable.as_os_str())?;
    for argument in args {
        output.push(0x20);
        output.extend(quote_windows_argument(argument)?);
    }
    output.push(0);
    if output.len() > 32_767 {
        return Err(crate::CodexCliEditorError::CommandLineTooLong);
    }
    Ok(output)
}

#[cfg(windows)]
mod windows {
    use std::mem::{size_of, zeroed};
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use std::ptr::{null, null_mut};
    use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

    use winapi::shared::minwindef::{BOOL, DWORD, FALSE, TRUE};
    use winapi::um::consoleapi::SetConsoleCtrlHandler;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::jobapi2::{
        AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject,
    };
    use winapi::um::processthreadsapi::{
        CreateProcessW, GetExitCodeProcess, PROCESS_INFORMATION, ResumeThread, STARTUPINFOW,
        TerminateProcess,
    };
    use winapi::um::synchapi::WaitForSingleObject;
    use winapi::um::winbase::{CREATE_SUSPENDED, INFINITE, WAIT_FAILED};
    use winapi::um::wincon::{
        CTRL_BREAK_EVENT, CTRL_C_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT,
    };
    use winapi::um::winnt::{
        HANDLE, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JobObjectExtendedLimitInformation,
    };

    use super::{OsString, build_windows_command_line};
    use crate::{CodexCliEditorError, Result};
    pub(super) const CLOSE_HANDLER_WAIT_MS: u32 = 3_000;
    static ACTIVE_CHILD: AtomicPtr<core::ffi::c_void> = AtomicPtr::new(null_mut());
    static ACTIVE_CLOSE_HANDLERS: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "system" fn console_control_handler(control: DWORD) -> BOOL {
        match control {
            CTRL_C_EVENT | CTRL_BREAK_EVENT => TRUE,
            CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT => {
                ACTIVE_CLOSE_HANDLERS.fetch_add(1, Ordering::SeqCst);
                let child = ACTIVE_CHILD.load(Ordering::SeqCst);
                if !child.is_null() {
                    // SAFETY: ConsoleHandler::drop waits for every handler that observed the
                    // process handle before the owned handle can close.
                    unsafe { WaitForSingleObject(child, CLOSE_HANDLER_WAIT_MS) };
                }
                ACTIVE_CLOSE_HANDLERS.fetch_sub(1, Ordering::SeqCst);
                TRUE
            }
            _ => FALSE,
        }
    }
    struct ConsoleHandler;

    impl Drop for ConsoleHandler {
        fn drop(&mut self) {
            ACTIVE_CHILD.store(null_mut(), Ordering::SeqCst);
            // SAFETY: unregisters the same static handler registered for this launch.
            unsafe { SetConsoleCtrlHandler(Some(console_control_handler), FALSE) };
            while ACTIVE_CLOSE_HANDLERS.load(Ordering::SeqCst) != 0 {
                std::thread::yield_now();
            }
        }
    }
    struct Handle(HANDLE);

    impl Drop for Handle {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: this wrapper owns the valid handle.
                unsafe { CloseHandle(self.0) };
            }
        }
    }

    struct SuspendedChildGuard {
        handle: HANDLE,
        armed: bool,
    }

    impl Drop for SuspendedChildGuard {
        fn drop(&mut self) {
            if self.armed {
                // SAFETY: the guard is armed only while the created child remains suspended and
                // the owning process handle is valid.
                unsafe {
                    TerminateProcess(self.handle, 125);
                    WaitForSingleObject(self.handle, CLOSE_HANDLER_WAIT_MS);
                }
            }
        }
    }

    fn non_verbatim_argv0(path: &Path) -> std::path::PathBuf {
        let value = path.to_string_lossy();
        if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
            return std::path::PathBuf::from(format!(r"\\{rest}"));
        }
        if let Some(rest) = value.strip_prefix(r"\\?\") {
            return std::path::PathBuf::from(rest);
        }
        path.to_path_buf()
    }

    fn api_error(api: &'static str) -> CodexCliEditorError {
        CodexCliEditorError::WindowsApi {
            api,
            source: std::io::Error::last_os_error(),
        }
    }

    pub(super) fn run_native(executable: &Path, args: &[OsString]) -> Result<i32> {
        let canonical = executable
            .canonicalize()
            .map_err(|source| CodexCliEditorError::io(executable, source))?;
        let application: Vec<u16> = canonical.as_os_str().encode_wide().chain([0]).collect();
        let argv0 = non_verbatim_argv0(&canonical);
        let mut command_line = build_windows_command_line(&argv0, args)?;
        let mut startup: STARTUPINFOW = unsafe { zeroed() };
        startup.cb = size_of::<STARTUPINFOW>() as u32;
        let mut process: PROCESS_INFORMATION = unsafe { zeroed() };

        // SAFETY: all pointers reference live, correctly sized buffers. The application path is
        // absolute, and command_line is mutable and NUL terminated as CreateProcessW requires.
        let created = unsafe {
            CreateProcessW(
                application.as_ptr(),
                command_line.as_mut_ptr(),
                null_mut(),
                null_mut(),
                TRUE,
                CREATE_SUSPENDED,
                null_mut(),
                null(),
                &mut startup,
                &mut process,
            )
        };
        if created == 0 {
            return Err(api_error("CreateProcessW"));
        }
        let process_handle = Handle(process.hProcess);
        let thread_handle = Handle(process.hThread);
        let mut suspended_child = SuspendedChildGuard {
            handle: process_handle.0,
            armed: true,
        };

        // SAFETY: null security and name pointers request an unnamed job with default security.
        let raw_job = unsafe { CreateJobObjectW(null_mut(), null()) };
        if raw_job.is_null() {
            return Err(api_error("CreateJobObjectW"));
        }
        let job_handle = Handle(raw_job);
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: limits has the exact structure and length required for this info class.
        if unsafe {
            SetInformationJobObject(
                job_handle.0,
                JobObjectExtendedLimitInformation,
                &raw mut limits as *mut _,
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        } == 0
        {
            return Err(api_error("SetInformationJobObject"));
        }
        // SAFETY: both handles are valid and the child remains suspended.
        if unsafe { AssignProcessToJobObject(job_handle.0, process_handle.0) } == 0 {
            return Err(api_error("AssignProcessToJobObject"));
        }
        ACTIVE_CHILD.store(process_handle.0, Ordering::Release);
        // SAFETY: the handler is a static function and remains registered until the child exits.
        if unsafe { SetConsoleCtrlHandler(Some(console_control_handler), TRUE) } == 0 {
            ACTIVE_CHILD.store(null_mut(), Ordering::Release);
            return Err(api_error("SetConsoleCtrlHandler"));
        }
        let _console_handler = ConsoleHandler;

        // SAFETY: the primary thread is valid and currently suspended.
        if unsafe { ResumeThread(thread_handle.0) } == u32::MAX {
            return Err(api_error("ResumeThread"));
        }
        suspended_child.armed = false;

        // SAFETY: process handle remains valid until this function returns.
        let wait = unsafe { WaitForSingleObject(process_handle.0, INFINITE) };
        if wait == WAIT_FAILED {
            return Err(api_error("WaitForSingleObject"));
        }
        let mut exit_code = 1u32;
        // SAFETY: process has exited and exit_code is a writable u32.
        if unsafe { GetExitCodeProcess(process_handle.0, &mut exit_code) } == 0 {
            return Err(api_error("GetExitCodeProcess"));
        }
        Ok(exit_code as i32)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

    use super::{build_windows_command_line, quote_windows_argument};

    fn rendered(value: &str) -> String {
        String::from_utf16(&quote_windows_argument(value.as_ref()).unwrap()).unwrap()
    }

    #[test]
    fn quotes_windows_arguments_using_crt_rules() {
        assert_eq!(rendered("plain"), "plain");
        assert_eq!(rendered(""), "\"\"");
        assert_eq!(rendered("two words"), "\"two words\"");
        assert_eq!(rendered("a\\\"b"), "\"a\\\\\\\"b\"");
        assert_eq!(rendered("ends with \\"), "\"ends with \\\\\"");
        assert_eq!(rendered("&|<>^%"), "&|<>^%");
    }

    #[cfg(windows)]
    #[test]
    fn rejects_nul_and_overlong_command_lines() {
        use std::os::windows::ffi::OsStringExt;
        let with_nul = OsString::from_wide(&[0x61, 0, 0x62]);
        assert!(matches!(
            quote_windows_argument(&with_nul),
            Err(crate::CodexCliEditorError::ArgumentNul)
        ));
        let oversized = OsString::from("a".repeat(32_768));
        assert!(matches!(
            build_windows_command_line(Path::new(r"C:\tool.exe"), &[oversized]),
            Err(crate::CodexCliEditorError::CommandLineTooLong)
        ));
    }

    #[cfg(windows)]
    #[test]
    fn unicode_argument_encodes_actual_utf16() {
        let value = String::from_utf16(&[0x0633, 0x0644, 0x0627, 0x0645]).unwrap();
        let units = quote_windows_argument(value.as_ref()).unwrap();
        assert_eq!(units, vec![0x0633, 0x0644, 0x0627, 0x0645]);
    }

    #[cfg(windows)]
    #[test]
    fn close_handler_budget_matches_contract() {
        assert_eq!(super::windows::CLOSE_HANDLER_WAIT_MS, 3_000);
    }

    #[cfg(windows)]
    #[test]
    fn launches_native_child_and_preserves_exit_code() {
        let current = std::env::current_exe().expect("test executable");
        let code = super::run_native(&current, &[OsString::from("--help")]).expect("native child");
        assert_eq!(code, 0);
    }

    #[test]
    fn builds_nul_terminated_command_line() {
        let line = build_windows_command_line(
            Path::new(r"C:\Program Files\Codex\codex.exe"),
            &[OsString::from("--flag"), OsString::from("two words")],
        )
        .unwrap();
        assert_eq!(*line.last().unwrap(), 0);
        let rendered = String::from_utf16(&line[..line.len() - 1]).unwrap();
        assert_eq!(
            rendered,
            r#""C:\Program Files\Codex\codex.exe" --flag "two words""#
        );
    }
}
